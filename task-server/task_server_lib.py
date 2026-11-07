#!/usr/bin/env python3
"""singlefs 的统一任务服务（task-server）共用的那一份：路径、依赖、错误类型、放大重试、环境变量名单、编译目录、分片、身份、经 unix socket 调服务，连同整套的自证。

一切跑项目代码的任务（轻、重、CPU、GPU）都经这个服务起：同一个队、按到达次序、不另设延迟（records/2026-10-30-编译绕过内存总上限.md 的用户定案与形态）。
任务只由主 agent 交（客户端 task-server/task；人手同样用它）；子 agent 只用 status、log 看结局（用户：「现在需要发给 task-server的任务 都不能自己启动」）；挑机器、定内存额度、要不要 GPU、分哪张卡、线程数、分不分片都由服务定，提交时不收这几样。

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
  两台对称：不读认台的角色变量；卡的台标签是 HOSTS 里的地址，服务照 host_addresses（同一份共用取法的 resolve）
  认出自己这一台，把台标签映到它记尝试的 local / peer（task_server.py 文件头 ②「GPU」）。
  取不到的（整份判不过、自己认不出、没写 HOSTS）交原因与出路：NFS 共享的「挂载」一项判不过，control install、start、check 判多机配置时退 78。
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
              这一次 ssh 超时（NFS_SSH_SECONDS）先当「慢」：隔 NFS_RECHECK_PAUSE_SECONDS 秒复查一次（总共两次，peer_shell_rechecked），复查通了照通算，慢的那一句交给
              nfs_share_report 的 slow（服务只记进服务日志：不推提醒、不当第二台用不了）；复查还超时或连不上才算不过，那一句写「连不上第二台（ssh 超时 N 秒，隔 M 秒复查仍不通…）」；
              头一次就是连不上（ssh 退 255 而不是超时）的不复查，照写「连不上第二台（ssh 退 255：…）」（第二台 load average 约 60 时一次 30 秒超时、同一刻手跑 stat -f 秒回 nfs；
              弄坏开关 peer-health-single-timeout-reported 下头一次超时就算连不上）；
    两向探针  本机往 <探针目录>/.task-server-nfs-probe 写一个随机记号、第二台 cat 回来逐字相同；第二台往 <探针目录>/.task-server-nfs-probe-peer 写一个随机记号、本机读回逐字相同；
              读完两个文件都删。探针目录：根下的 .git 是目录（根是 git 工作树）时是 <根>/.git，不在项目根下留条目（仓根新条目钩子会报）；别的根是根本身（nfs_probe_directory）。
  没写这一键、写法不对、哪个根哪一项不过：服务不起（Scheduler 建起之前判，退 78），一句话写明哪个根、哪一项、为什么，出路指 task-server/README.md「## NFS 共享」。
  control check 判四项；control install 起之前判前三项、回读到 /health 回 ok 之后判四项；control start 起之前判前三项、回读之后判四项；不过都退 78。
  服务起来之后每 NFS_RECHECK_SECONDS（task_server.py）复核前三项（两向探针只在起来时做）：第二台没挂上的根，当第二台此刻用不了（task_server.py 文件头 ②「机器」）、写告警；
  /health 的 nfs 一栏是每个根的四项结论（两向探针取起来时那一次）；服务自己盯健康那一路（task_server.py 文件头 ⑥）每一轮读这一份，前三项不过就发消息给有任务在排、在跑的会话。
  第二台照同一绝对路径读写共享根（仓也在共享根下）：第二台在同一棵树、同一个绝对路径上跑，工作目录与命令里的路径有不在共享根下的提交时就拒（下面「挑机器」）。

库（${状态目录}/task-server.sqlite3，标准库 sqlite3、WAL，文件 600）：只有服务读写它。表 meta、tasks、attempts、history、item_rates、operations、alerts，
  每一行的正文是一段 JSON（record 列），另有一列 signature = HMAC-SHA256(签名密钥, 表名、行号、state、session、command_key、record 用换行连起来)，小写十六进制。
  服务每次读回都核：核不过的行不起、不认、写告警（kind=record-tampered），任务记成 record-tampered 收尾。会话号只进库与仓外的日志，不进仓。
  决定关联的那一列 task_id（attempts_of、children_of 照它挑尝试与子任务）不在签名里：读的时候与正文的 task_id 比、子任务另比 parent_id，对不上就是核不过（TS-05，
  records/2026-11-01-task-server深入审核.md）；停单元、删编译目录、推分片遇到核不过的行一律不动、回 409 或记 record-tampered。
  状态转换（TS-01、TS-03）：任务行只经 Store.change 改——一个事务里读当前那一行、核、判、只并进要改的那几栏，不拿调用方手里的旧字典整行覆盖；结了局的任务不回到别的状态。
  起一次尝试是一个事务（Store.claim_attempt）。业务唯一：attempts 的 (task_id, 正文里的 number)、tasks 的 (正文里的 parent_id, role, shard_index)；
  已有的库违反它服务不起、退 78，出路是停着服务时 control clear-queue（这一版不认那种库）。
  history 表放钉值（下面「资源记录」）；导进来的峰值与用时那几行定额度与预计用时不读。
  item_rates 表放按项的两个速率（下面「资源记录」的预计用时）：每（入口脚本、项名）一行，task_id 0、state=ITEM_RATE_STATE、session 空、command_key 是 item_rate_key
  （业务唯一），正文 {entry, item, seconds_per_judged, seconds_per_reused, judged_from, reused_from}：两个速率是每条几秒（没记过是 null），*_from 记它出自哪一次
  （{task, attempt, seconds, judged, reused, at}）。control clear-queue 不动它。

错误类型（封闭的表；每次尝试记一种，任务的结局是最后一次尝试的那一种或服务判出的那一种；wait 的退出码照右列，「照命令」是命令自己的退出码）：
  succeeded               0      命令退 0
  command-failed          照命令  命令自己退非 0：表里没有的退出码一律落在这里，不吞掉（本机的 255 也在这里）
  memory-cap-hit          250    撞了这一条自己的上限（内存包装的 250）；放大额度重试，见「重试」
  memory-cap-at-maximum   236    撞上限、已到两台最大：再乘 1.5 就到能派的每台 slice 总上限，不再加；wait 的末行写撞了哪台、到了多少、改什么再重投（下面「撞顶到两台最大」）；
                                 或排队那一刻这一条在每一台上的记账都超过那台的可用上限（task_server.py 文件头 ②「预算」）：一次都不起，说明写记账、两台可用上限与记账从哪来
  squeezed-out            254    被总上限挤掉（内存包装的 254）；同一上限重排
  gpu-memory-short        245    显存不够（内存包装的 245）；等显存再试一次
  timed-out               253    超时（内存包装的 253）
  wrapper-refused         251    内存包装用法错或 scope 起不来（251）
  run-condition-unmet     78     运行条件不满足：命令或清场包装退 78（清场回读没过），或 GPU 任务而多机配置没点名能用的卡（服务判），
                                 或钉了上限而多机配置的 SLICE_MEMORY_CAP_BY_HOST 此刻读不到或写坏了（服务判，下面「资源记录」的钉值），
                                 或分片的一片认不出它所在那一台在 HOSTS 里的地址（服务判，下面「分片」②），
                                 或能派的每一台这一轮都量不到余量、总上限不比余量大或钉值不大于 0（服务判，task_server.py 文件头 ②「余量」）；服务判的几种一次都不起，说明写原因与出路
  not-run                 77     本次未跑（命令退 77）
  peer-run-failed         255    第二台没跑成：派到第二台、multi-host-run.sh 退 255、日志里没有第二台那条命令的结束行（下面「第二台的退出码」）
  prune-server-unavailable 75    prune-server 不可用：命令自己退 75（scripts/prune-server/prune_server_client.py 的 EXIT_UNAVAILABLE；第二台的照结束行的码）；不重试，
                                 服务另起一次 python3 prune-server/control check，不通写告警 prune-server-unavailable（只报，不起、不停 prune-server）
  card-fault              237    清场判出卡坏（尝试日志里有 GPU_QUIESCE step=quiesce result=card-fault）；另写一条告警，出路是主 agent 弹窗请用户重置那张卡。
                                 坏卡清单：清场那一行的 faulty-cards=<台>:<序号>[,<台>:<序号>…]（scripts/crash/gpu-quiesce-run.sh 文件头「输出」；台是 HOSTS 里的地址，
                                 可以是 IPv6，从右边切一刀；序号 unknown 的是总线读不出、归不到哪张卡的，不算进清单），日志里取最后一行清场 card-fault 行
                                 （card_fault_cards）。记尝试时存进这一次尝试的 faulty_cards（["<台>:<序号>", …]）。停用的、告警与结局句点名的是这几张，
                                 不是这一次分到的那张（attempt 的 card）：清场查的是分到的卡，查卡结果点名的才是坏的（task_server.py 的 settle_attempt）。
                                 清单取不到（没有这一栏、写法认不出、只有 unknown）：一张都不停用、不重排，照 card-fault 收尾，告警与结局句写明取不到坏卡清单
  launcher-lost           238    对不回来：尝试的单元不在了、又没留结果
  cancelled               239    被取消（提交它的会话经客户端 cancel，回读过单元停了）
  record-tampered         240    库被改过：读回校验不过（绕过服务直接写进库的行，或写进之后被改过）；一次都不起
  never-fits              241    哪台都等多久也放不下（scripts/hosts/host-dispatch.py 的 judge_host 判 never），或点名第二台而第二台此刻用不了
  unit-start-failed       235    尝试的单元起不来（systemd-run 退非 0）
  resubmit-required       232    服务规则改了，还在排的这一条照现行提交规则不再收（服务起来时再核：环境变量、包装与资源参数、编译目录）；
                                 一次都没起，wait 末行写不合哪一条，提交方改了重交
  stalled                 233    卡住：起来后先宽限 STALL_GRACE_SECONDS（10 分钟）不判，之后日志距上一次增长满 STALL_SECONDS（10 分钟）、这段时间 CPU 也几乎没涨，或已跑超过记录用时的 5 倍而日志满 10 分钟没涨（不看 CPU），服务停了它（两台都判）；不重试，看 task log 改了重投。
                                 日志与 CPU 都满窗口不涨、而这个窗口里每一轮都有进程在 D 状态（不可中断的 IO 等待）的不记这一种、不停：卡在 IO 上，在等盘或 NFS
                                 （task_server.py 文件头 ①「卡在 IO 上」；本机读进程状态，第二台读 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_IO_WAIT），连着满 30 分钟报要处置的提醒 io-wait；读不到 D 状态的照上面判
                                 要卡的、这一次尝试分到了卡，日志与 CPU 都满窗口不涨、而这个窗口里每一轮卡上都有它的计算负载、卡的利用率有一轮不为 0 的同样不记这一种、不停：GPU 在算
                                 （task_server.py 文件头 ①「卡上有它的计算进程」；本机读 nvidia-smi 的计算进程与 utilization.gpu，第二台与量不到的当有负载；利用率读不到的照有负载免停、
                                 依据写「利用率读不到」），连着满 30 分钟报要处置的提醒 gpu-busy-quiet；卡上有它的进程而利用率一个窗口里每一轮都是 0 的（占着卡不干活）门槛另提到
                                 CARD_IDLE_STALL_CPU_CORES（1 个核，task_server.py）：命令自己的 CPU 这一窗口平均不到 1 个核的（CPU 上轮询死等）停、记这一种，说明写日志多久没涨、
                                 卡的利用率与命令自己的 CPU 三样读数；不少于 1 个核的是在 CPU 上算，照跑；读不到利用率的照上面判；
                                 有一轮卡上没有它的负载、没分到卡的照上面判；远超记录用时的那一条照停，不看 D 状态也不看卡
                                 判卡住那一刻这一次 scope 的内存峰值已到上限的 STALL_AT_CAP_FRACTION（98%，task_server.py）的不记这一种：顶在上限上卡在回收里，
                                 停了之后记 memory-cap-hit、照「重试」乘 1.5 重排（钉值的照钉值规矩打回 memory-cap-at-maximum），说明写「顶在上限上卡住」；第二台的峰值读不到，照记 stalled
  over-two-hours          242    跑满两小时打回：各次尝试在跑的时长累加（排队等的不算；撞了内存上限重来的计时归零，只从最后一次撞上限之后那一次起算）满 RUN_LIMIT_SECONDS（7200 秒，
                                 task_server.py），服务停了它（两台都判）；分片的各片按自己的尝试各自累计。豁免只有提交时的门禁（commit_gate_exempt）：起的是 COMMIT_GATE_ENTRIES
                                 （提交时的六道门禁与 scripts/gate/gate-staged.sh）之一、不带 --check 与 --item、范围声明 SINGLEFS_RUN_SCOPE 的「为什么」里写了「提交前」；
                                 带「长跑已确认」标识的（task requeue 重进的，task_server.py 文件头 ⑦）不判这一条；带「整包已确认」标识的照判。不重试，原号不自动重排：提交它的会话核过范围、确是要跑这么久的，
                                 照 wait 打的那一行 task-server/task requeue <号> --long-run-confirmed --items <N> --tasks <M> --why '<…>' 重进队尾；不对的改了范围照常重交
  shard-plan-refused      234    分片的计划那一步退 0 而它写的计划读不通（下面「分片」：plan.json 不在、不是 JSON、片数超过 MAXIMUM_SHARDS、键写错）
  退出码对类型：0 succeeded；75 prune-server-unavailable；77 not-run；78 run-condition-unmet 或 card-fault（看日志）；245、250、251、253、254 照上表（内存包装的退出码表在
  scripts/hosts/run-with-memory-cap.sh 文件头「退出码」）；255 在第二台是 peer-run-failed、在本机是 command-failed；别的非 0 都是 command-failed（252 也在这里：内存包装判出命令退出之后停了自己的 scope、
  回读还有进程，run-with-memory-cap.sh 文件头「退出码」，stderr 列着留下的进程）。
  第二台的退出码（records/2026-10-31-task-server巡检.md 问题 79）：multi-host-run.sh 在第二台那条命令等到结局时往日志打
  「MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>」（PEER_COMMAND_END_LINE，取最后一行）：有这一行照这个码（第二台那条命令自己的、经那边内存包装的）
  按上面对类型，255 记 command-failed、不重排，multi-host-run.sh 之后收尾退的码不盖掉它（尝试另记 peer_runner_exit_code）；没有这一行而退 255 才是 peer-run-failed。
  命令自己也退得出 232–242 这几个码，wait 的末行写着类型，分得开。
  疑似卡住（不是结局，不进上表）：
  （疑似卡住）            231    任务照跑：wait 打一行「TASK_SERVER_SUSPECT task=<号> attempt=<第几次> host=<本机|第二台> …」与依据、退 231，不打 TASK_SERVER_RESULT；
                                 再起 wait 接着等（同一段只报一次）；判法在 task_server.py 文件头 ①「疑似卡住」（SUSPECT_MARK、SUSPECT_EXIT）
  （需要处置 needs-attention） 230    任务照旧在排或在跑（换卡重排已经跑成的，任务已有结局）：wait 打一行
                                 「TASK_SERVER_ATTENTION task=<号> type=needs-attention kind=<种类> basis=<依据的数> why=<原因> next=<出路>」（值照 shlex 引；ATTENTION_MARK、ATTENTION_TYPE），
                                 任务还没结局的退 230（ATTENTION_EXIT），不打 TASK_SERVER_RESULT，再起 wait 接着等；任务已有结局的这一行打在 TASK_SERVER_REASON 之前，退码仍是任务的。
                                 十二种 kind（ATTENTION_KINDS）：queued-too-long 排队过久，overrun 跑到记录用时的很多倍（日志在涨服务不停它），
                                 oversized-budget 记账不小于某一台的整片界（预算贴着整台，照排），long-run-idle-cores 跑满一小时而平均在用核数不到给的线程数四分之一（照跑），
                                 fills-any-cap 同一命令键最近三次峰值都贴着各自上限（按给的上限开并行），
                                 quiet-log-busy 日志长期不涨而服务没停它（CPU 在涨或读不到；没有记录用时的也报），io-wait 卡在 IO 上而服务没停它
                                 （日志没涨的每一轮都有进程在 D 状态，连着满 30 分钟），gpu-busy-quiet 卡上有这一次尝试的计算进程、日志与 CPU 都不涨而服务没停它
                                 （分到了卡、每一轮卡上都有它的计算负载、卡的利用率有一轮不为 0 或读不到，连着满 30 分钟），card-replaced 清场判出卡坏、
                                 换别的卡重排（结局可能是 succeeded），cleanup-pending 清场没收尾（资源占用保留），shard-advance-failed 分片父任务推不动，
                                 workspace-write-interrupted 改主工作区的命令没正常结束、可能写了一半（只报有结局的任务，退码仍是任务的；点名要核的文件或补丁目录）；
                                 每种的判法、档与阈值（QUEUE_ATTENTION_MINUTES、OVERRUN_ATTENTION_FACTOR、QUIET_ATTENTION_MINUTES、IO_WAIT_ATTENTION_MINUTES、
                                 GPU_BUSY_QUIET_ATTENTION_MINUTES）在 task_server.py 文件头 ①「要处置的提醒」；next= 的头一段照下面「异常表」那一行的修改意见。
                                 同一条任务同一种提醒只叫醒一次，档跨过下一档（阈值的两倍、四倍…）才再叫；服务另直接发消息给提交它的会话（task_server.py 文件头 ⑥，同一份记账）；服务健康不归 wait，服务自己盯、发消息
  打回的理由（用户：「任务出问题  返回给对应的对话时候 要写按照分类明白理由」）：wait 在末行 TASK_SERVER_RESULT 之前打一行
    「TASK_SERVER_REASON type=<错误类型> <要素>… why=<一句人读得懂的原因> next=<出路>」（值照 shlex 引；reason_line），task status 给了号且有结局的同样打。
    每种类型要带的要素在 REASON_ELEMENTS：memory-cap-hit 带 cap=（上限）peak=（峰值）；memory-cap-at-maximum 带 host= reached= peak=；squeezed-out 带 host= cap= by=（同台在跑、
    把它挤掉的那几条，task_server.py 的 finish_attempt 记的 squeezed_by）；resubmit-required 带 rule=（不合的那一条）；never-fits 带 judged=（哪台为什么放不下）；
    要卡而卡用不上的（gpu-memory-short、run-condition-unmet、card-fault）带 card= quiesce=（日志里清场那一行）；命令自己退的那几种带 last_line=（日志最后一行）。
    next= 照下面「异常表」里这一种结局那一行的修改意见（anomaly_advice）；memory-cap-at-maximum 前面另接撞了哪台、到了多少、峰值多少（maximum_next_step）。
  每次尝试记：机器（local / peer）、上限、卡、线程数、编译目录、退出码、错误类型、起止时刻、用时、内存峰值（scope 进程用的内存 anon 加 shmem 见过的最大，下面「退出时的内存峰值」之后「峰值的口径」；量不到记 null，不记 0）、
  平均在用核数（scope 的 cpu.stat 的 usage_usec 除以用时）、显存峰值（本机 GPU 任务：nvidia-smi 读到的这一条子孙进程用量之和的最大值）、日志路径、单元名，
  另记编译段峰值 build_phase_peak（下面「资源记录」的编译族）。

异常表（封闭的表；一行一种，栏用「｜」隔开：种类｜动作｜判据｜修改意见｜没处置时）：服务判出的每一种异常——提交时拒与提交时警告、交了待确认、
  错误类型表里除 succeeded 之外的每一种结局、要处置的提醒（ATTENTION_KINDS）、疑似卡住、结局没人取——都在这里有一行，怎么判、怎么办、没人管时怎么办只照这一行。
  动作是 ANOMALY_ACTIONS 四样之一：提交时拒（不进库、不跑）、打回（记结局、交给提交它的会话改了重交）、提醒照跑（服务不停、不改任务，只把这一行的修改意见交给会话）、
  服务自己修（重排、换卡、取消这类，修不成的照打回办）。要处置的提醒、结局（打回与服务自己修）、疑似卡住、结局没人取、没确认被取消、确认的数对不上与客户端读不到收件地址，
  修改意见就是 wait 打的 next= 那一句与发给会话的正文「出路：」那一行的头一段（{task} 换成任务号，后面另接这一回的具体数）：服务照 anomaly_advice 从这张表取，代码里不另写一份。
  提交时拒的其余八种与两种提交时警告，拒绝说明在判它的那一处写（要点名这一回的变量、路径、包或那几条结局），表里那一行写的是同一条出路，给人查；
  每一句点名这一种要核的东西，不写「看 status、等不起就取消改小重交」这种通用句。
  没处置时：时长判的提醒照档每翻一倍升一档再推；不随时长涨的（oversized-budget、fills-any-cap、long-run-idle-cores、cleanup-pending、card-replaced、shard-advance-failed，
  ANOMALY_REPUSHED_KINDS）同一档推过之后满 ANOMALY_REPUSH_SECONDS（30 分钟）还在的再推一回、之后每翻一倍再推，正文写前几次推了什么、此刻还差什么；
  结局满 30 分钟没交给会话的照 unclaimed-outcome 那一行再推。自证 anomaly-table-matches-kinds 读这一段与 ATTENTION_KINDS、OUTCOME_TYPES、SUBMIT_REFUSAL_KINDS 比，对不上判红。
  environment-refused｜提交时拒｜调用方环境里有白名单外的变量，或环境里、命令里设了退役的变量（RETIRED_VARIABLES）｜去掉拒绝说明点名的那个变量再交；命令真要它的写成 task-server/task submit -- env <名>=<值> <命令…>，退役的照说明里的新写法改｜不进库，没有下一档
  nested-wrapper｜提交时拒｜命令外层或 bash -c 里套了内存包装、派发器、清场或线程包装，或写了 --memory、--memory-cap、--threads、--peer、--local-only、SINGLEFS_THREAD_CAP｜删掉拒绝说明点名的那一层或那个参数再交：内存、线程、机器与卡都由服务定｜不进库，没有下一档
  target-directory-in-repository｜提交时拒｜调用方给的 CARGO_TARGET_DIR（环境里或命令里）指到仓里｜去掉它再交，编译目录由服务给；真要自己的给仓外、共享根下的路径｜不进库，没有下一档
  outside-shared-path｜提交时拒｜工作目录或命令里的绝对路径有不在多机配置 NFS_SHARED_ROOTS 任何一段下的（/tmp 下的同样算）｜把拒绝说明点名的那个路径改放共享根下（草稿放工作根 ~/.local/share/singlefs/work/ 下），再交｜不进库，没有下一档
  shared-roots-outside｜提交时拒｜交给命令的 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT 有不在共享根下的｜照点名的那个变量改多机配置的 SHARED_DATA_ROOT，或去掉任务里带的那个变量，再交｜不进库，没有下一档
  verdict-library-unserved｜提交时拒｜命令点到 .claude/gate.d/verdict-library-scripts.tsv 里带判定库而第三列还没写 prune-server 的一行，或那张表读不了、写坏了｜把点名的那一族接进 prune-server、第三列翻成 prune-server（表写坏的先改对那一行），再交｜不进库，没有下一档
  temporary-service-heavy｜提交时拒｜临时 prune-server 包装里剥出的原命令会编 cargo 或要卡｜去掉包装重交：验证包的 cargo test 对真服务判｜不进库，没有下一档
  whole-workspace-cargo｜提交时拒｜命令位置上会编的 cargo 带 --workspace / --all，或不带 -p 而照清单的走法落到虚拟清单｜点名要编的包 -p <包>（验证包写 -p singlefs-verification）；整仓要跑的走 scripts/gate/merge-and-verify.sh 或提交时的门禁｜不进库，没有下一档
  inbox-missing｜提交时拒｜客户端读不到会话的收件地址（环境里没有 CLAUDE_CODE_MESSAGING_SOCKET 与 CLAUDE_CODE_MESSAGING_TOKEN）｜从 Claude Code 会话的 Bash 里交（那里的环境带收件地址）；子 agent 不交任务，写进报告交回主 agent｜不进库，没有下一档
  whole-package-unconfirmed｜提交时拒｜命令是整包测试（cargo test 不点名 --test、--bin、带测试名的 --lib，或带 --lib --tests、--workspace；crash-point-replay.sh --check 三格全点；不给补丁目录的 merge-and-verify.sh）而没带 --whole-package-confirmed，或带了而确认文件的形式不合（不是共享根下读得到的绝对路径、缺 ①②③ 哪一段、哪一段不到 12 个字）；提交时的门禁豁免｜改成只点名改到的测试目标与测试函数；确实要整包的照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍（①改到的文件与函数、②引到它们的测试目标与函数及 grep 命中、③为什么只点名②不够）写进共享根下的一份文件，带 --whole-package-confirmed <确认文件> 重交；带了照样受两小时限｜不进库，没有下一档
  requeue-counts-mismatch｜提交时拒｜task requeue 给的项数或在排在跑的条数与服务此刻算的不同（与 task confirm 同一个核法，task_server.py 的 counts_mismatch）｜照拒绝说明打的确认单核这条命令的范围（点的项、测试目标）与这个会话在排在跑的那几条：范围对了照服务算的两个数再 task-server/task requeue {task} --long-run-confirmed --items <N> --tasks <M> --why '<为什么要跑这么久、范围为什么是这些>'，不对的改了范围照常 task-server/task submit 重交｜不进队，没有下一档
  confirm-counts-mismatch｜提交时拒｜task confirm 给的项数或在排在跑的条数与服务此刻算的不同｜照拒绝说明打的确认单核这条命令的范围（点的项、测试目标）与这个会话在排在跑的那几条：范围对了照服务算的两个数再 task-server/task confirm {task}，不对的 task-server/task cancel {task}、改了重交｜照 unconfirmed 那一行
  repository-target-bypass｜提醒照跑｜命令里的词、或命令位置上起的本仓脚本正文指向仓根 target/ 下的二进制，绕过服务给的编译目录｜编译交给服务给的编译目录，二进制取这一趟编出来的位置（CARGO_TARGET_DIR 下）；入口脚本里删掉仓根 target/ 的路径，改完 task-server/task cancel {task}、重交｜提交回复、日志头与 status 三处照打，不另推
  unclaimed-results｜提醒照跑｜交任务时会话名下有父任务的结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）没交给它｜task-server/task wait <号> 把点名的几条结局取走、照结局那一行的出路办完，再 task-server/task clean <号>｜每交一条照打；那几条结局照 unclaimed-outcome 那一行再推
  unconfirmed｜服务自己修｜交了而满 CONFIRM_TIMEOUT_SECONDS（30 分钟）没 task-server/task confirm（状态 awaiting-confirmation）｜核确认单（task-server/task status {task}）里的范围与项数：对的重交、照确认单 task-server/task confirm；不要了不用管｜服务取消（结局 cancelled）、推一条「没确认，已取消」；那一条结局没人取照 unclaimed-outcome 再推
  queued-too-long｜提醒照跑｜在排、从提交（重排的从上一次尝试结束）起排满 QUEUE_ATTENTION_MINUTES（30 分钟），排队原因已判出来｜task-server/task status {task} 核「在等：…」点名的是哪一台、哪一样（余量、整片封组、卡、编译目录占盘、第二台用不了）：等余量的核这一条的上限是不是比它该要的大，第二台用不了的看 python3 task-server/control check｜每翻一倍（60、120 分钟…）升一档再推
  overrun｜提醒照跑｜在跑、有记录用时，这一次已跑到 max(OVERRUN_ATTENTION_FACTOR × 记录用时, 宽限 10 分钟)｜task-server/task log {task} 核最后几行还在不在推进（项数、轮数在涨），对照这一趟的范围是不是比记录用时那一次大；不推进的 task-server/task cancel {task}、修了重交｜每翻一倍升一档再推
  quiet-log-busy｜提醒照跑｜在跑、日志满 QUIET_ATTENTION_MINUTES（30 分钟）没涨，服务因 CPU 在涨或读不到 CPU 没停它｜task-server/task log {task} 核最后一行停在哪一步，task-server/task status {task} 核在用核数：一直满核而日志不动的多半在回收里打转或死循环，核那一步是不是把输出攒着不打｜每翻一倍升一档再推
  io-wait｜提醒照跑｜在跑、日志没涨的每一轮都有进程在 D 状态，连着满 IO_WAIT_ATTENTION_MINUTES（30 分钟）｜核这一次尝试所在那台的盘与 NFS：/proc/pressure/io 的 some avg60、谁在大量写盘（同台在跑的全目标编译这类）、第二台上共享根的 NFS 挂载在不在、rpc 等不等得回；IO 缓过来它自己接着跑｜每翻一倍升一档再推
  gpu-busy-quiet｜提醒照跑｜在跑、分到了卡，日志与 CPU 都不涨而每一轮卡上都有它的计算进程、卡的利用率有一轮不为 0 或读不到，连着满 GPU_BUSY_QUIET_ATTENTION_MINUTES（30 分钟）；利用率一个窗口里都是 0 的照卡住停，不报这一条｜核那张卡的利用率与显存在不在动（nvidia-smi），task-server/task log {task} 核停在哪一步；还在算就等，卡死了 task-server/task cancel {task}｜每翻一倍升一档再推
  card-replaced｜提醒照跑｜尝试结局 card-fault 而清场点名了坏卡，服务已停用它、换别的卡重排（结局可能是 succeeded）｜主 agent 弹窗请用户重置日志里 GPU_QUIESCE 那一行点名的那张卡；任务不用重交｜又判出一张坏卡升一档；同一档推过满 30 分钟再推，之后每翻一倍再推
  cleanup-pending｜提醒照跑｜命令已退出而清场回读还有进程或读不了，服务保留这一次的资源与显卡占用｜task-server/task log {task} 看留下的是哪几条命令行，核实脚本有没有不等子进程就退（后台 &、setsid 出去的没等齐）；改好之后重交，那几个进程停掉之后服务自己重查收尾，不要删进程号、退出码或清理状态文件来放行｜每次尝试一档；同一档推过满 30 分钟再推，之后每翻一倍再推
  shard-advance-failed｜提醒照跑｜分片或估算的父任务推下一步连着出错｜看 journalctl --user -u singlefs-task-server.service 里「推下一步时出错」那一句前后的栈，核计划 plan.json 与各片的产物目录读不读得通；修好之前任务 {task} 不往下走，等不起 task-server/task cancel {task}｜推不动的每一回一档；同一档推过满 30 分钟再推，之后每翻一倍再推
  workspace-write-interrupted｜提醒照跑｜改主工作区的命令以撞上限、被挤掉、取消、卡住被停、对不回来这类结局收尾或被信号杀，可能写了一半｜核这条命令改的文件或补丁目录（依据里点名）与 git status --short、git diff --stat：没写全的照原样补齐或退回，核过再重交｜随结局推一回；结局没人取照 unclaimed-outcome 再推
  oversized-budget｜提醒照跑｜排队那一刻在某一台上的记账不小于那台的整片界（可用上限的 WHOLE_FRACTION，85%）而不超过可用上限｜核实这一趟的范围（点的项、测试目标）、分片与同时起的个数；确要这么多就照跑，不要的 task-server/task cancel {task}、收小或分片再交｜同一档推过满 30 分钟还在排、在跑的再推，之后每翻一倍再推
  long-run-idle-cores｜提醒照跑｜在跑满 LONG_RUN_ATTENTION_SECONDS（一小时），到此刻平均在用核数不到给的线程数的 LONG_RUN_IDLE_CORE_FRACTION（四分之一）｜核实脚本是否串行（该切片并行的照 .claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」）、范围是否点多（点的项、测试目标）；要改就 task-server/task cancel {task}、改了重交｜每次尝试一档；同一档推过满 30 分钟还在跑的再推，之后每翻一倍再推
  fills-any-cap｜提醒照跑｜同一命令键最近 FILLS_CAP_RUNS（3）次跑成的峰值都不低于各自上限的 FILLS_CAP_FRACTION（90%）；撞了上限的那几次不算（撞的那一次峰值本来就贴着上限）｜这条命令按给的上限开并行，核实：同时起的个数要以这一趟的范围为界（不超过要判的单位数），不照服务交的上限（TASK_SERVER_MEMORY_CAP_BYTES）定｜同一档推过满 30 分钟还在排、在跑的再推，之后每翻一倍再推
  suspected-stall｜提醒照跑｜宽限过后整个单元的 CPU 满 STALL_SECONDS（10 分钟）平均不到一个核的 1%，要卡的这段时间卡上也没有它的计算负载｜task-server/task log {task} 核停在哪一步、在等什么（锁、子进程、网络）；还在算就不用管，真卡住了 task-server/task cancel {task}｜一段只推一回；CPU 又涨了算这一段结束，再静下来是下一段
  unclaimed-outcome｜服务自己修｜父任务的结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）没交给提交它的会话（消息没送到，也没起 wait）｜task-server/task wait {task} 取走结局、照结局那一行的出路办完，再 task-server/task clean {task}｜每翻一倍（60、120 分钟…）再推一回，推到取走为止
  command-failed｜打回｜命令自己退非 0（表外的退出码都在这里）｜task-server/task log {task} 核最后一屏是哪一条断言、哪一步报错，改了命令或代码再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  memory-cap-hit｜服务自己修｜撞了这一条自己的上限（内存包装退 250），或判卡住那一刻顶在上限上｜服务乘 1.5 重排、至多 MEMORY_CAP_RETRIES 次；次数用完打回的，task-server/task status {task} 核各次的峰值是不是一次比一次大（规模变大、内存泄漏），把要的内存降下来再重交｜重排的不推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  memory-cap-at-maximum｜打回｜撞上限而再乘 1.5 就到能派的每台的可用上限，或排队那一刻记账超过每一台的可用上限｜核实这一趟的范围（点的项、测试目标）、分片与同时起的个数，收小或分片再交（task-server/task submit）｜随结局推一回；没人取照 unclaimed-outcome 再推
  squeezed-out｜服务自己修｜被 slice 的总上限挤掉（内存包装退 254）｜服务同一上限重排、至多 SQUEEZE_RETRIES 次；用完打回的，核 by= 点名的同台那几条是不是记账偏小，等它们跑完再重交｜重排的不推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  gpu-memory-short｜服务自己修｜显存不够（内存包装退 245）｜服务同一上限重排一次；还不够的核 card= 那张卡上谁占着显存（nvidia-smi），等它空出来再重交｜重排的不推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  timed-out｜打回｜内存包装的限时到了（253）｜task-server/task log {task} 核卡在哪一步，把这一趟的范围拆小再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  wrapper-refused｜打回｜内存包装用法错或 scope 起不来（251）｜task-server/task log {task} 看内存包装打的最后那一句，照它修好（systemd --user 在不在、slice 配了没有）再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  run-condition-unmet｜打回｜运行条件不满足：命令或清场包装退 78、要卡而没点名能用的卡、钉了上限而配置读不到、能派的每一台都量不到余量｜照 why= 那一句点名的设备或配置（多机配置那一项、卡、清场回读）修好再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  not-run｜打回｜命令报本次未跑（77）｜task-server/task log {task} 核它报缺的是哪一项前提（设备、工具、输入），补齐再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  peer-run-failed｜服务自己修｜第二台那一趟没跑成（连不上、ssh 被重置、那边的准备没过）｜服务同一上限重排、至多 PEER_RUN_RETRIES 次；用完打回的看 python3 task-server/control check 与 /health 的 peer 一栏，修好第二台再重交｜重排的不推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  prune-server-unavailable｜打回｜命令自己退 75：prune-server 不可用｜看 python3 prune-server/control check 那一句修好 prune-server 再重交；不要改成走本地库｜随结局推一回；没人取照 unclaimed-outcome 再推
  card-fault｜服务自己修｜清场判出卡坏（尝试日志里有 GPU_QUIESCE step=quiesce result=card-fault）｜主 agent 弹窗请用户重置 quiesce= 那一行点名的卡；服务已停用它、换卡重排，坏卡清单取不到的核还有没有好卡（多机配置 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）｜换卡重排的照 card-replaced 推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  launcher-lost｜打回｜尝试的单元不在了、又没留结果｜看 journalctl --user -u <unit= 点名的单元> 核单元是谁停的、有没有被 OOM 杀，照原样重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  cancelled｜打回｜提交它的会话取消了，或交了满 30 分钟没确认被服务取消｜要跑就重交；没确认被取消的照确认单核过范围再 task-server/task confirm｜随结局推一回；没人取照 unclaimed-outcome 再推
  record-tampered｜打回｜库里这条记录校验不过｜查是谁绕过服务直接写了库（告警记录点名哪一行），照 task-server/task submit 重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  never-fits｜打回｜哪台都等多久也放不下，或点名第二台而它用不了｜核 judged= 那一句点名的那一台为什么放不下（峰值比 slice 总上限大、卡凑不够张数），把命令要的内存或卡收小再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  unit-start-failed｜打回｜尝试的单元起不来（systemd-run 退非 0）｜看 systemctl --user status 与 journalctl --user，核用户级 systemd 在不在、单元名有没有撞上，修好再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  shard-plan-refused｜打回｜分片的计划那一步退 0 而它写的计划读不通｜核 plan.json 在不在、是不是 JSON、片数超没超 MAXIMUM_SHARDS、键写没写错（文件头「分片」），改了计划那一步再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  stalled｜打回｜宽限过后日志与 CPU 都满 STALL_SECONDS（10 分钟）不涨（分到了卡、卡上有它的进程而利用率每一轮都是 0 的，命令自己的 CPU 这一窗口平均不到 CARD_IDLE_STALL_CPU_CORES（1 个核）就算不涨），或跑过记录用时 5 倍而日志满 10 分钟没涨，服务停了它｜task-server/task log {task} 核最后一行停在哪一步、那一步在等什么（锁、子进程、网络）、是不是把输出攒着不打；改了再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  over-two-hours｜打回｜不是提交时的门禁，从第一次尝试起跑各次尝试在跑的时长累计满 RUN_LIMIT_SECONDS（两小时），服务停了它；带「长跑已确认」标识的不判｜核两件事：范围对不对（点名范围：--item、--check 少几格、点名测试目标；给补丁目录；或照服务分片；整轮的留到提交时），这条是不是确实要跑这么久；不对的改了范围照常 task-server/task submit 重交；对的照 wait 那一行抄 task-server/task requeue {task} --long-run-confirmed --items <N> --tasks <M> --why '<为什么要跑这么久、范围为什么是这些>'（以原命令、原范围声明重进队尾，记长跑已确认，不再按两小时打回）｜随结局推一回；会话不确认就停在这一种，没人取照 unclaimed-outcome 再推
  resubmit-required｜打回｜服务规则改了，还在排的这一条照现行提交规则不再收｜照 rule= 那一条（环境变量、包装、编译目录）改了再重交｜随结局推一回；没人取照 unclaimed-outcome 再推

重试（内存不够就放大额度；每一次尝试都留记录，追加不覆盖）：
  memory-cap-hit：上限 × CAP_GROWTH_FACTOR（1.5，按 MiB 向上取整）重排、不钉台（哪台装得下去哪台，照「挑机器」），不超过那台的总上限（那一台 slice 的总上限：那一台的内存包装照多机配置 SLICE_MEMORY_CAP_BY_HOST 里它那一项设成 singlefs-heavy.slice 的 MemoryMax，服务读两台的余量报告；第二台另取 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项的小者）；
    乘 1.5 会到或超过那台总上限的不重排（用户定「涨到总上限就打回」：slice 里总挂着一点回收不掉的内核内存，记满一整台永远凑不齐；
    这里的总上限照上限收到的那个数比：那台总上限减那台此刻的余量，task_server.py 的 usable_maxima 与 host_capacity，余量量不到的那台当查不到；task_server.py 文件头 ②「余量」）：
    有总上限比乘 1.5 之后还大的另一台就换过去（上限取乘 1.5 之后的），没有这样的一台记 memory-cap-at-maximum 打回。
    至多重试 MEMORY_CAP_RETRIES（16）次，次数用完结局记 memory-cap-hit。
  squeezed-out：同一上限重排，至多 SQUEEZE_RETRIES（3）次。gpu-memory-short：同一上限重排一次（GPU_SHORT_RETRIES）。
  peer-run-failed（第二台没跑成：连不上、ssh 被重置、那边的准备没过）：同一上限重排、不钉台，至多 PEER_RUN_RETRIES（2）次。别的类型不重试。
  重排的任务照任务号回到队里原来的位置（比它后到的排在它后面）。
撞顶到两台最大（用户：「如果超过上限那么直接报错，返回给agent错误信息。要求修改重新投。」）：结局 memory-cap-at-maximum，wait 退 236，
  末行在 TASK_SERVER_RESULT 那几栏后面接 host=<本机|第二台> reached=<到顶那一次的上限> next=<改什么再重投>（maximum_next_step；分片里那一片撞的另带 shard=<片号>/<片数>）。
  next= 与发给会话的正文同一句（用户：「8848 撞了就打回 问到底任务范围和分片是不是核实的」）：撞了哪台、到了多少、峰值多少，修改意见是核实这一趟的范围（点的项、测试目标）、
  分片与同时起的个数，收小或分片再交；记账超过每一台可用上限、一次都没起的（task_server.py 文件头 ②「预算」）reached= 写那份记账、峰值写预测的峰值。

资源记录（用户：「长任务的用时 资源用量 需要存储下来  每次启动器启动的时候需要参照这个来配置资源」）：
  只认服务自己库里的尝试记录（用户：「可以去掉」）：定额度与预计用时不读内存包装的峰值表，也不读 history 表里导进来的峰值与用时那几行
    （history 表里定额度只认钉值那一种）。
  命令键：命令的词先剥掉不改跑法的环境变量前缀（key_words：起头的 env 与紧跟的「名=值」里名在 KEY_NEUTRAL_VARIABLES——
    SINGLEFS_MULTI_HOST_CONFIG、SINGLEFS_RUN_SCOPE、BINDGEN_EXTRA_CLANG_ARGS、LANG——的剥掉，别的赋值与 env 照留；
    一个都没剥掉的照原样），再照 scripts/hosts/memory_peaks_lib.py 的 command_key 换算（RUN_WITH_MEMORY_CAP_KEY 给了照它）。提交、钉值、分片子任务都用这一个
    （任务 2103 带 env 前缀、落在另一个键上，没吃到钉值）。弄坏开关 environment-prefix-kept 下不剥。退役变量的赋值在提交时就拒了（下面「环境变量」），不进命令键。
  定额度（task_server.py 的 recorded_prediction）：同一命令键最近 RECORD_WINDOW 次量到的内存峰值（服务自己记的尝试，按结束时刻从新到旧；量不到的那几次不算、
    不占名额）取最大；上限 = 最大值 × 1.5，按 MiB 向上取整。
    服务自己的尝试只用跑完的（失败的、提前退出的尝试，它的峰值不拿来定下一次的额度；RECORD_COMPLETED_OUTCOMES、RECORD_CAP_HIT_OUTCOMES）：
      退 0 的（succeeded）记它量到的峰值；撞了这一条自己上限的（memory-cap-hit，到顶收尾的那一次同样）记「至少要到这个上限」，按它的上限算（真峰值不小于它）。
      所以下一次同一命令从撞过的最大上限 × 1.5 起，不从小的再爬一遍：只撞过 2048M 的，下一次 3072M 起；
      撞过 2048M、后来在 3072M 跑完的，下一次取跑完那次的峰值 × 1.5（向上取整），不低于 3072M。
      别的结局（command-failed、cancelled、launcher-lost、squeezed-out，连同错误类型表里别的几种）的峰值不进定额度、不占名额；尝试记录照存，status 照显示。
      理由：提前失败的那一次峰值偏小，不代表这条命令要多少（records/2026-10-30-编译绕过内存总上限.md「服务修复两件合起来验」#8：脚本提前退 1 的那一次峰值 42 MiB，
      修好之后同一条命令照它定 63M，一路撞上限放大到 728M）。
    服务自己有这条命令结束了的尝试、一次跑完的都没有的：照下面「没有记录的命令」办（4G），另以判红那一次的峰值当下界：最近一次命令自己退非 0（command-failed）
      而量到了峰值的尝试，上限与记账不低于那个峰值 × 1.5（按 MiB 向上取整；尝试记录 prediction 记 FAILED_PEAK_HISTORY；task_server.py 的 failed_peak_floor）。
      理由：任务 2045 第 1 次上限 13824M、峰值 3.34 GiB、退 1，同一命令键的任务 2159 从 4096M 起、跑 8007 秒撞顶。弄坏开关 failed-peak-ignored 下不看判红的峰值。
    没有记录的命令（用户：「没记录的先记小，撞了再放大」）：服务的记录里一次跑完的都没有的，
    预测、上限与记账都取 UNRECORDED_MEMORY_DEFAULT_BYTES（4G，上限写成 4096M；不乘 1.5；task_server.py 的 unrecorded_prediction）。撞了 250 照「重试」乘 1.5 重排（4096M → 6144M → 9216M …），乘 1.5 会到那台总上限的换总上限更大的一台、没有就记
    memory-cap-at-maximum 打回，同一套；撞过的那几次进了服务的记录，同一条命令下一回按记录定。缺省取 4G 的理由（主 agent 定的，records/2026-10-30-编译绕过内存总上限.md
    「问题清单的去向」#6）：经服务跑过的任务 1–8 实测峰值 10 MiB 到 1.71 GiB；常见命令已从峰值表导入 3321 条记录，没有记录的多是小命令；
    按 8G 的缺省乘 1.5（12288M）记账时，几条没记录的命令就把本机 slice 记满，后到的被挤去第二台（records 同一份「批 3b 交回与验证」任务 5、6）。
  钉值（用户：「变异的内存上限直接按照 根目录设置的50gb走」「你这边也直接 将数据库设置为50gb 让他自己算一次多少个并发」「50 这个是读取的配置 不是随便给的」）：
    history 表里 state=kind=pin 的一行按命令键记「这条命令的上限取多机配置 SLICE_MEMORY_CAP_BY_HOST 里派到的那一台那一项」（PIN_STATE、PIN_SOURCE），不记字面的数；
    定额度先认它，预测、上限与记账都取每次现读的配置值减那一台此刻的余量：判哪一台放不放得下就取哪一台那一项（task_server.py 的 booked_at_cap、pinned_on_host 按台换），
    减完不大于 0 的那一台不派（control pin-cap 当场拒，task_server.py 文件头 ②「内存」的钉值），尝试记录 prediction 记
    PINNED_HISTORY；学到的峰值不顶掉它；撞了上限不放大，照 memory-cap-at-maximum 打回。
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
  有记录的照记录、不设 4G 底（用户：「有记录的照记录、去掉 4G 底」；records/2026-10-31-task-server巡检.md 批 91）：没有记录的照上面先记 4G；
    服务自己有真实峰值记录的（定额度那一段算出来的，prediction 记 RECORD_HISTORY）上限照记录 × 1.5（按 MiB 向上取整），记账照上限，不往 4G 抬
    （用户：「这个交给 task-server callback记账，如果有真实的使用数据 按照真实的使用数据来」；任务 5953、5995–5999 预测 17 MiB、照记 4G 排了 1.5–2 小时）；
    撞了上限照「重试」乘 1.5 重排，重排那一次照乘出来的上限，不兜到 4G；撞过上限的命令再交一遍，撞过的那一次的上限进记录窗口，头一次照它 × 1.5 起（下面「按近几次最高起跑」）。
    钉值照旧。弄坏开关 recorded-cap-floored 下照改前把钉值之外低于 MEMORY_FLOOR_BYTES（4G）的上限与记账抬到 4G（有记录的头一次、重排、撞过上限再交、改工作区的首趟都兜；
    task_server.py 的 memory_floor_applied、workspace_writer_headroom）。
  改主工作区的命令首趟给足（records/2026-10-31-task-server巡检.md 批 77；任务 8367：hook-swap.py 带 --verify 照同键窗口 × 1.5 只给 857M，撞了、打回、不重排，换上做了一半）：
    task_server.py 的 modifies_workspace 认的命令不是重排的那一次，上限与记账不低于 (这把键近 RECORD_WINDOW 次最高 + 它换上之后要起的验证命令
    那把键近 RECORD_WINDOW 次最高) × 1.5（不设 4G 底），比上面几段定出的大才换（prediction 记 WORKSPACE_WRITER_HISTORY；task_server.py 的 workspace_writer_headroom、verify_command_keys）。
    撞了上限照旧不重排（重排会把写了一半的再写一遍）。弄坏开关 workspace-writer-headroom-skipped 下照改前只看同键窗口。
  编译族（records/2026-10-31-task-server巡检.md 批 60：会编的 cargo 同键没记录只从 4G 起、一撞再撞）：提交时给每条记一段编译族 build_family（task_server.py 的 build_family_of）：
    会编的 cargo（compiles）取它起的每一条会编的 cargo 的包（-p / --package，去重排序）、档（--release 与 --profile release 同一档，没写是 dev）、特性
    （--features / -F 去重排序，--all-features、--no-default-features）与 --target（cargo_build_family；子命令、测试目标名、测试名筛选、`--` 之后的 libtest 参数不进，
    一个包都没点名的记 cargo 起的那个目录相对它所在工作树的根）；命令位置上起的入口脚本文件头前 DECLARED_HEADER_LINES 行里写了
    「# task-server-cargo-family: <cargo 参数>…」（DECLARED_BUILD_FAMILY_MARK）的照写的参数算（declared_build_family；单独 --selftest、带 --list-items / --help / -h 的不算），
    例：入口文件头写「# task-server-cargo-family: -p singlefs-verification --release --features gpu」，与直接起的 cargo test --release -p singlefs-verification --features gpu 同一族。
    编哪个包由命令参数定的入口，声明里写取自参数的词（filled_family_words）：「$<n>」是脚本后面的第 n 个词（从 1 数，照原样数，选项与它的值各占一个），
    「$<选项名>」（$ 后面接以 - 起头的名字）是那个选项的值（认「选项 值」「选项=值」，`--` 之后的不认），提交时照这一条命令填上再算族；
    有一个填不上（没有那个词、值是空的或以 - 起头）的这条命令不归族，照没记录的 4G 起。例：scripts/mutation/prove-red.sh --copy <副本> <crate> <变异名…> 写「-p $3」，
    变异名每条都不同（同键从来没有记录，任务 7896 第 1、2 次 4096M、6144M 都撞了上限），按 <crate> 归进同一族、从同族近几次的编译段峰值起步。弄坏开关 family-arguments-unfilled 下不填、照字面算。
    声明由管那份入口的会话加，服务只读。每次尝试记下任务的 build_family；同键一次跑完的都没有的，与撞上限之后的每次重排，都照同族（别的命令键的也算）已结束的尝试里，
    跑成的与撞了上限的那几次的编译段峰值（尝试的 build_phase_peak），按结束时刻从新到旧取最近 RECORD_WINDOW 个、取最大 × 1.5 起步（与「按近几次最高起跑」、兄弟片同一个窗口，
    同一个取窗口的函数 window_maximum；比照原来定的上限大才换；prediction 记 BUILD_FAMILY_HISTORY），有自己记录的照自己的（task_server.py 的 build_family_peak）。
    编译段峰值：日志里 cargo 打出「Finished … target(s) in …」那一行（CARGO_FINISHED_PATTERN，取头一行）之前按轮采到的进程内存（anon 加 shmem，下面「峰值的口径」）的最大；
    本机由 attempt_runner.py 采（它文件头 ③，写进 result.json 的 build_phase_peak），第二台由 multi-host-run.sh 照它每一轮的读数采（它文件头 ⑤「编译段峰值」，
    打一行 PEER_BUILD_PHASE_PEAK_MARK），两台同一种量；见到那一行的那一轮不算。日志里没有那一行（编不过、不是 cargo）或那一行之前一轮都没采到的这一栏记 null、不借。
    整趟的峰值（memory_peak）与撞了上限那一次的上限只给同一命令键自己（定额度、按近几次最高、兄弟片）：族里各条命令编的是同一份，跑测试那一段要的内存各不相同，
    借整趟的会把一条单测目标按族里跑得最大的那一条记账。弄坏开关 family-lends-run-peak 下族里照借整趟的峰值与撞了上限那一次的上限。
    能升能降：之后几次都编得小，大的那一次出了窗口就回落。分片的计划那一步与各片继承父任务的族；合并与估算那一步不继承（推的：它们读计划与各片的产物、列项，多半不编，
    继承了会照编译族的大峰值多记账）。弄坏开关 build-family-ignored 下不记、不看族，family-cap-hit-not-counted 下族里只收跑成的那几次的编译段峰值。
  同一计划共用峰值（同一批：同一计划的各片各一个键、各自从 4G 爬）：分片的一片不是重排的那一次时，同一父任务名下计划那一步与别的片已结束的尝试里，跑成的峰值、
    撞了上限的那一次的上限，按结束时刻取最近 RECORD_WINDOW（5）个、取最大 × 1.5（能升能降：之后五片都小，早先撞过的那一档出了窗口就回落），比这一片照上面定的上限大才换（只往上抬，不压到没记录的 4G 以下；prediction 记 SHARD_FAMILY_HISTORY；task_server.py 的 shard_family_peak）。
    弄坏开关 shard-siblings-unshared 下各片照自己的键。
  按近几次最高起跑（用户 2026-11-05：「对高频的 cargo test --release 和 crash-point-replay 这类命令，直接按历史峰值起跑，不从 4G 一级一级爬」；
    同一条 mutation-tables 命令任务 7349 从 4096M 爬到 20736M、7416 与它同时在跑又从 4096M 爬一遍，各撞四五次）：上面几段定出上限、兜过下限之后，
    新提交的头一次与撞上限之后的每次重排都再看这把键服务自己记着的近 RECORD_WINDOW 次里最高的一次（撞过的上限、跑成的峰值，按结束时刻从新到旧取，
    与定额度同一个窗口），× 1.5 比定出的上限大就换上（prediction 记 KEY_HISTORY_PEAK_HISTORY；task_server.py 的 key_recent_peak、history_raised_prediction）；
    重排的那一次另照编译族与同一计划的兄弟片往上抬（撞了上限说明自己的记录不够用；编译族与兄弟片同样只取近 RECORD_WINDOW 个，兄弟片也收撞过的上限，编译族只收编译段峰值）。起之前复核现算一次（同键别的任务这期间撞到的更高一档也看得到）。
    能升能降（用户：「资源的使用记录 服务要有升降的能力 不能只升不降」）：窗口里刚撞上去的那一档新提交与重排都看得到；更早的出了窗口就不再算，后面几次都跑得小，起点跟着降。
    没任何记录的照旧 4G，有真实峰值记录的不兜那一条照旧。弄坏开关 history-peak-ignored 下不看（头一次只照定额度那一段、重排只照自己那一次 × 1.5 一级级乘），
    history-peak-unwindowed 下照改前不截近几次、全部记录里取最大（一次大峰值永远压着后面，只升不降）。
  预计用时（task_server.py 的 estimated_seconds）：排队时 status 报它（estimated_seconds），连同这个数是照哪种算法得的（estimate_basis：ESTIMATE_BASIS_COMMAND 按命令、
    ESTIMATE_BASIS_ITEMS 按项、ESTIMATE_BASIS_NONE 没有；task_server.py 的 estimate_of），客户端 task status <号> 照它打 ESTIMATE_BASIS_SENTENCES 里对应的那一句
    （弄坏开关 estimate-basis-unreported 下服务不交这个字段、一句说明都打不出，estimate-basis-sentence-fixed 下客户端不看它、一律打按命令那一句）；
    封了组的那台照预计用时判短任务插不插空、挡路的还要跑多久（task_server.py 文件头 ②「整片与碎片按台交替」的插空）。两种算法：
    按项（用户：「目前的数据库支持这种级别的参数识别码」「B可以。重算的代价和排队的代价不成正比。」）：命令起的是 ITEM_ENTRIES 里的五条入口之一（item_entry，剥法同分片）、
      点了 --item、又不分片的（分片的照「分片」那一套，不先起估算子任务，预计用时照下面「别的」那一种）：提交时任务记 state=estimating、不进队，先建一条估算子任务（role=ESTIMATE_ROLE，create_child 那一套：
      各自定额度、挑台、排队、记峰值与用时，上限公式与别的任务同一套；命令是 estimate_command：同一条入口接 --list-items --with-keys、原命令的参数照原样；
      命令键原文是父任务的键接 ESTIMATE_KEY_MARK；不要卡、不编）。它的日志里每项一行「ITEM_PLAN item=<项名> to_judge=<要现判的条数> reusable=<能复用的条数>」
      （ITEM_PLAN_LINE，同一项几行取最后一行），收尾时记进那一次尝试的 item_plan；子任务有了结局父任务才改记 queued、照原来的任务号进队排
      （task_server.py 的 drive_item_estimate），子任务跑成而有 ITEM_PLAN 的把各项那一行记进父任务的 item_plan；子任务没成（退非 0 这类）或一行 ITEM_PLAN 都没打的，
      父任务照样进队、item_plan 记 null、没有预计用时，不卡住（弄坏开关 estimate-failure-holds-parent 下父任务一直在估算）。估算时取消父任务：估算子任务照取消整件那样停、回读，
      停了父任务记 cancelled（task_server.py 的 cancel_estimating）。弄坏开关 item-estimate-skipped 下照改前不估、提交了就进队。
      每次尝试收尾（task_server.py 的 record_item_rates）：命令起的是那五条入口之一的（点没点 --item、分不分片都算），从整份日志读每项一行
      「ITEM_DONE item=<项名> seconds=<这一项的挂钟秒数> judged=<现判条数> reused=<复用条数> exit=<这一项的退出码>」（ITEM_DONE_LINE；几项并行时 seconds 是各项自己的挂钟），
      exit=0 的按（入口脚本、项名）在 item_rates 表里记最近一次的「每现判一条几秒」（seconds_per_judged）与「每复用一条几秒」（seconds_per_reused）：
      judged 为 0 的只更新复用的速率（seconds ÷ reused），reused 为 0 的只更新现判的（seconds ÷ judged），两样都不为 0 的只更新现判的：
      (seconds − reused × 已记的复用速率) ÷ judged（不小于 0），还没有复用速率的取 seconds ÷ judged（把这一项的挂钟都算给现判，宁可估长）；两样都是 0 的不记。
      exit 不是 0 的那一项不记（可能是提前退）。用时是运行记录，只记在服务自己的库里，不进 prune-server。弄坏开关 item-done-unrecorded 下不读不记。
      预计用时 = 点到的各项 to_judge × seconds_per_judged + reusable × seconds_per_reused 之和（task_server.py 的 item_estimate）；有一项缺速率
      （item_rates 里没有这一项、要现判而没有现判速率、要复用而没有复用速率、item_plan 是 null 或里面没有这一项）整条交 None、不插空
      （弄坏开关 item-rate-missing-skipped 下缺速率的那一项当 0 秒照加、item_plan 是 null 的照旧 None，item-estimate-by-command 下照改前按命令那一种算）。
    别的（没点 --item 的、不是那五条入口的、估算子任务自己、分片的计划与合并两步和各片）：服务自己库里同一命令键（与内存记录同一把键：路径、条目照词折过，
      --check 的格名照留进键）、最近一次命令自己跑到退出的尝试的用时（取结束时刻最晚的一次）；没有交 None。命令自己跑到退出：结局是 ESTIMATE_COMPLETED_OUTCOMES
      （succeeded、command-failed）之一，被信号杀的 command-failed（退 128 + 信号号，task_server.py 的 SIGNAL_EXIT_CODES）不算；撞上限、超时、卡住被停、被挤掉、取消、
      跑满两小时这些服务判出或打断的结局不在里面（任务 9730 跑约 1807 秒退 1，同一条再交只照退 0 的那一次报 80 秒）。弄坏开关 estimate-only-succeeded 下只认 succeeded。任务 7585 与 7447 只差补丁目录：
      内存照同键记录预测了 30.9 GiB，预计用时照逐词相同取不到，没有预计用时就不插空、封组挡住后面（弄坏开关 estimate-by-exact-command 下照改前只认命令逐词相同的）。
      分片的计划、各片与合并（SHARD_ROLES）也走这一种：每一步是一个子任务，命令就是父任务的命令，命令键各是这件的键接那一步一段（shard_child_key），
      所以取的是这一步自己最近一次跑成的用时：计划一份、合并一份、各片按片号和片数各一份（片号不同的不混）。各片本来可以照片里的项按上面那一种相加，
      可是计划那一步不交每片的项（plan.json 每片只认 label，见「分片」①），按项做不到，照这一片自己的历史估。
      弄坏开关 shard-plan-merge-unestimated 下计划与合并没有预计用时，shard-pieces-unestimated 下各片没有（改前分片那三步一律没有）。
  退出时的内存峰值（任务 5012：跑 1 秒左右的片本机记 null，第二台的计划、合并与一片也是 null；按轮采在 scope 收尾之前一次都没采到）：每一次尝试另经环境给内存包装
    RUN_WITH_MEMORY_CAP_PEAK_OUTPUT（PEAK_OUTPUT_VARIABLE）= <尝试目录>/EXIT_PEAK_FILE_NAME，包装在命令退出之后、scope 收尾之前读一次 scope 的 memory.peak 写进去；
    本机 attempt_runner.py 照说明里的 exit_peak_file 读，按轮一次都没采到才记进 memory_peak；两样都没有仍是 null。弄坏开关 peak-read-only-by-sampling 下不给、不读（改前只靠按轮采）。
  峰值的口径（两台同一种量）：按轮采的是进程用的内存——内存包装那一层 scope 的 memory.stat 的 anon 加 shmem（scripts/hosts/memory_peaks_lib.py 的 process_memory_bytes），
    留见过的最大；页缓存与内核挂在 scope 上的不算（任务 8853 编验证包：anon 921M、file 22163M，memory.peak 顶到上限 50176M，照 memory.peak 记，编译族照它 × 1.5 起步就记满一台）。
    本机在 attempt_runner.py 的采样里读（它文件头 ③），第二台在 multi-host-run.sh 每轮那一次 ssh 里读（它文件头 ⑤「峰值」）。退出时那一份是 scope 的 memory.peak（进程退了之后
    读不到进程内存），只给比一轮还短、按轮一次都没采到的命令兜底。弄坏开关 memory-peak-with-page-cache 下本机照改前按轮读 memory.peak、与退出时那一份取大的。
  峰值口径一栏（records/2026-10-31-task-server巡检.md 批 93：批 92 之前记的 memory_peak 含页缓存，编译族起步被抬到 45–50G）：服务记下结局的每一次尝试另写一栏
    peak_metric = PROCESS_MEMORY_PEAK_METRIC（「进程内存」，上面那一种口径；编译段峰值 build_phase_peak 同一种口径）。定额度（recorded_measurements、key_recent_peak）、编译族与兄弟片（attempt_measurement）、
    判红那一次峰值的下界（failed_peak_floor）与 fills-any-cap（task_server.py 文件头 ①「要处置的提醒」）只读这一栏是它的行（task_server.py 的 current_metric）；
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
  这层包装只给脚本自证用：剥出的原命令有会编的 cargo 或要卡的，提交时就拒，出路是去掉包装重交（task_server.py 文件头 ②「提交时拒」④）；编译目录、分卡与清场只照外层那一条给。
  不带 --server-binary 的包装自己要现编 prune-server（prune_server_client_selftest.py 的 resolve_binary 只在环境里有任务服务给的编译目录 CARGO_TARGET_DIR 时现编），
  所以外层这一条按会编的算（temporary_service_builds_server，task_server.py 的 compiles）：两台都由服务给编译目录；带了 --server-binary、或命令前的赋值与任务环境里
  给了 PRUNE_SERVER_BINARY 的不编、不给。弄坏开关 temporary-service-build-unrecognized 下不认（改前的样子：派本机不给编译目录、包装退 2，派第二台由 multi-host-run.sh 照例给）。
  自带临时服务的两份脚本（OWN_SERVER_SELFTESTS）单独起的自证同理：prune_server_client_selftest.py 的 --selftest 与 --selftest-families、prune-server-load-simulation.py 的 --selftest
  不带 --server-binary、又没有 PRUNE_SERVER_BINARY 时，自己走同一个 resolve_binary 现编，也按会编的算（own_server_selftest_split）：两台都由服务给编译目录，带了二进制的照旧不编。
  任务 7069、7070、7071 都记成 "compiles": false 而退 0（别的会话读任务记录量到的），应是因为它们落在第二台、第二台照例由 multi-host-run.sh 给了编译目录；派本机不给编译目录、resolve_binary 退 2 是按源码推的，没有任务实测过。
  弄坏开关 temporary-service-own-server-unrecognized 下这两份不认。
  CPU 不记账（用户：「每次都开31 按照内存查。 现在核心的问题是内存不足」）：准入只看内存与显存。
  准入按上限记账（用户：「当然， 谁超了杀谁 不要连累别人」）：在跑的与新起的都按上限记（记账 = 上限，收到那台总上限减那台此刻的余量为止，钉值同样收；峰值本身装不下的照原样），每台在跑任务的上限之和不超过那台 slice 的总上限减那台此刻的余量；
    分片的片：记满上限放不下、预测放得下时，上限与记账一起压到那台此刻放得下的量（用户：「不能超过 内存上限」；task_server.py 的 shard_cap_fitted_to_room）：上限之和不超过总上限，超了只杀它自己。
    余量每台现量、按 1 GiB 向上取整，不是常量（task_server.py 文件头 ②「余量」，量法在 scripts/hosts/memory_peaks_lib.py 文件头「余量」）；
    还没涨到量的那一截照「上限 − 它此刻回收不掉的用量」算（task_server.py 的 booked_at_cap、server_pending），两台同一段：用量照这一轮读那一台余量的那一份报告
    （host-dispatch.py 的 probe_local / probe_peer，第二台是同一次 ssh）按 scope 量回来、照 scope 里进程环境的 ATTEMPT_LABEL_VARIABLE（每次尝试两台都给）对到尝试上，
    报告里没有那一条的 scope 记整份上限；
    第二台那一层内存包装不排队、不记账，记账只在服务这一处。

挑机器（task_server.py 的 candidate_hosts 与 place；本机的环境、库与配置第二台那一趟带不全，records/2026-10-30-编译绕过内存总上限.md 批 3b）：
  按余量均衡（用户：「两台或者多台的负载均衡也是一个重点」，选「三条都改，按余量均衡」）：不要卡的任务在派得过去的几台里挑放得下的
    （照 host-dispatch.py 的 judge_host，减去还没涨到量的那一截（排着的一律不留账，没满时限的照整片与碎片按台交替，满时限的排到最前、放不下时不挡后面，task_server.py 文件头 ②「满时限」）；预测用这一条存的那一份、余量用这一轮读的那一份，起之前现读复核一次，
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
    盘余量够，下面两段）与余量。改主工作区的工具（hook-swap.py、apply-writer-patch.py、stage-mine.py、scripts/kb/ 下的、
    不带 --skip-apply 的 merge-and-verify）与带 --write 的派得到第二台（改的就是同一棵树）；它们撞上限这类本会重排的结局照旧不重排（task_server.py 的 modifies_workspace）。
    不认合入锁（task_server.py 文件头 ②）：主工作树正在合入时照常派。
  第二台用不了要告警（task_server.py 的 note_peer）：连不上、余量查不成、多机配置判不过、挂载复核没过、盘余量不到下限（下一段）、那一趟没跑成（peer-run-failed），
  写告警 peer-unavailable，/health 的 peer 一栏写明原因，用不了满一段时间服务发「服务健康」给会话（task_server.py 文件头 ⑥）；哪一台此刻用不了，排队原因带上那一句。
  盘余量（records/2026-10-31-task-server巡检.md 问题 72：第二台根分区剩 353M 时任务 peer-run-failed；两台同一个下限）：服务读每台余量的那一次
    （task_server.py 的 Resources.snapshot，同一个缓存周期）另判那台编译目录所在盘（本机是 TASK_SERVER_BUILD_DIRECTORY 所在盘，第二台是 BUILD_ROOT_BY_HOST 里它那一项）的余量：
    报告里有 disk_available（字节，task_server.py 的 DISK_REPORT_KEY）照它，没有就照那一台的接口现量（本机读盘，第二台经同一条连法起 df -B1 --output=avail -- <编译根>）；
    不到 DISK_FLOOR_BYTES（50G）或查不成，那台这一轮用不了，原因写「<台>盘只剩 X（下限 Y）」，第二台的照上一段告警（盘不够这一种只写一条，余量的数变了不另写）、
    /health 的 peer 一栏写明；够了下一轮自己认回。
  分片的计划、各片与合并都不钉台，哪台由服务挑（各片第二台在前）。提交时不收挑机器的参数，没有「点名要第二台」的写法。
  同一件的兄弟片不避开彼此（用户：内存够就多跑）：放不放得下只看那一台的内存额度与空卡，同一台能同时跑几片；不按片号预先定台。
  GPU 任务不走这一条，照多机配置里卡的先后挑（task_server.py 文件头 ②「GPU」）：那一串里写在前面的第二台的卡先于写在后面的本机的卡。
线程（同上）：每次尝试的命令外面套 scripts/hosts/capped.sh <N>，N 是登记的每台默认重活线程数（.claude/heavy-thread-default，经 scripts/hosts/heavy_threads.py 取）；
  一律这一个数，提交时不收（SINGLEFS_THREAD_CAP 不在白名单里，带了当场拒）。第二台那边同一个数（双机两台各取这一个数）。
GPU（要不要卡由服务认，认出的一律套清场、分卡；cargo 特性、按名字认的入口与 python 入口的认法在 task_server.py 文件头 ②「GPU」）：
  按参数才要卡的脚本在自己的文件头声明：命令位置上起的脚本（bash / sh 起的、python3 起的、直接执行的；source 读进来的不算）文件头前 DECLARED_HEADER_LINES（60）行里
  有一行「# task-server-gpu-arguments: <词> [<词>…]」（DECLARED_GPU_ARGUMENTS_MARK），这一条命令的第一个非选项参数（不以 - 起头的头一个参数）等于其中一个词就要卡；
  写 * 是任何参数都要卡。单独 --selftest、带 --list-items / --help / -h 的不算。文件不在、不是普通文件、没有那一行照不要卡（declared_gpu_arguments）。
  例：e185-r3-segment-run.sh 文件头写「# task-server-gpu-arguments: host-cpu-per-submission recompute-disagreement」，bash scripts/experiment-apparatus/e185-r3-segment-run.sh
  host-cpu-per-submission 就要卡、别的段不要卡。编出来的二进制（命令位置上是 <编译目录>/release/<名字> 或 …/debug/<名字>）读仓里唯一那一份同名源
  （crates/ 或 scripts/experiment-apparatus/ 下任一层的 src/bin/<名字>.rs 或 src/bin/<名字>/main.rs）前同样行数里的「// task-server-gpu-arguments: <词>…」，判法相同（declared_gpu_binary）。按名字认的那几样（task_server.py 的 GPU_SCRIPT_NAMES、GPU_ENTRY_SCRIPT_NAMES、PYTHON_GPU_ENTRIES、GPU_SCRIPT_PATTERNS）照留。
  弄坏开关 gpu-argument-declaration-ignored 下不读这一行。
  要几张卡（张数；用户：「你需要测试的是 一张卡1gb 2张卡 三张卡 每张卡都能用来分片gc。没有卡报错。 有卡但是失败 报错加cpu兜底」；任务 7227 只分到一张）：
  要卡的这一条要同一台的几张，在源文件头声明，调用方照旧不写卡：命令位置上起的脚本文件头前 DECLARED_HEADER_LINES 行里写「# task-server-gpu-cards: <张数>」
  （DECLARED_GPU_CARDS_MARK）；编出来的二进制的同名源、与 cargo test / run / bench 点名（-p）的包里的源，前同样行数里写「// task-server-gpu-cards: <张数>」
  （DECLARED_SOURCE_GPU_CARDS_MARK）：点了 --test <名> 的读 tests/<名>.rs（或 tests/<名>/main.rs），只点 --lib 的读 src/lib.rs，别的读 src/lib.rs、src/main.rs、
  src/bin/ 与 tests/ 下的（cargo_card_sources；包目录照 Cargo.toml 的 [package] name 在 crates/ 与 scripts/experiment-apparatus/ 下找，package_directories）。
  几处都写了的取最大，都没写是一张（declared_gpu_cards）；写的不是一个正整数的不猜，服务记 run-condition-unmet、说明点名哪一份写了什么。
  例：crates/singlefs-background-gpu/tests/<目标>.rs 头几行写「// task-server-gpu-cards: 3」，cargo test -p singlefs-background-gpu --features tests-on-a-real-gpu --tests 就要同一台三张。
  服务照卡的优先级挑凑得出这么多张空卡的一台、整组分给它、整组清场；哪台都凑不出的记 never-fits（task_server.py 文件头 ②「GPU」）。弄坏开关 gpu-card-count-ignored 下一律一张。
续跑（用户 2026-09-26：跑超过 30 分钟的测试、装置、门禁阶段与脚本要能中断续跑）：脚本自己能续跑的，在文件头声明「# task-server-resumable: <词> [<词>…]」
  （DECLARED_RESUMABLE_MARK，读法与要卡的参数那一行相同：前 60 行、第一个非选项参数是其中一个词或写了 *；resumable_command，起的脚本照 command_word_index 剥掉
  env、nice、timeout、setsid、bash / sh / python3、capped.sh N 之后认）。被打断后重跑同一条命令就接着跑的才写这一行。
  一次尝试跑超过 LONG_RUN_SECONDS（1800 秒）、任务的结局不是 succeeded（取消的不算）、命令又没声明续跑的，结局说明前面写「这条跑了 N 分钟、没有续跑：重交会从头跑」
  （task_server.py 的 finished_preface）。弄坏开关 long-run-not-noted 下不写，resumable-declaration-ignored 下声明了也当没声明。

编译目录（用户 2026-10-23 定案「自己用完自己删」；records/2026-10-30-编译绕过内存总上限.md）：
  命令里命令位置上有会编的 cargo（.claude/hooks/lib_workspace_cargo_lock.py 的 cargo_invocations 与 BUILDING_SUBCOMMANDS 认，包装剥掉之后照认）、
  调用方没给编译目录的，本机这一次尝试设 CARGO_TARGET_DIR=<TASK_SERVER_BUILD_DIRECTORY>/<尝试标签>/target；任务记下结局就删它各次尝试的编译目录
  （用户：「服务build之后必须删除历史」；task_server.py 的 remove_finished_build_directories，服务起来时 sweep_finished_build_directories 把已有结局还留着的再扫一遍），
  wait 末行不带 build=。提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的留着现场：wait 末行带 build=<编译目录>，由提交它的会话用 task-server/task clean <号> 删。
  弄坏开关 finished-build-kept 下照改前一律留着等 task clean（起来时也不扫）。
  取消的不留产物（用户：「取消的任务要自己删掉产物 不要残留」「这个加入流程中 让task server去做」）：task cancel 回读到单元停了，服务在同一次取消里删这条任务
  （分片与估算的连同子任务）每一次尝试的编译目录与现场目录（下面「现场」），提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的也删，不再等 task clean；派到第二台的，这一趟第二台本地的 builds/<标签>/ 与 runs/ 下进程号、退出码、输出三个文件由 multi-host-run.sh 的 ⑥ 删，
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
  什么时候删：提交它的会话 task wait 取走结局（collected_at）之后过 SCENE_KEEP_AFTER_COLLECT_SECONDS（task_server.py，1 小时）由排队线程每一轮扫一遍删
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
  一件拆成几个子任务，各自是普通任务：各自按记录定额度（子任务的命令键是这件的键加「〈task-server 分片 <步> 第<片号>片共<片数>片〉」，经 RUN_WITH_MEMORY_CAP_KEY 交给内存包装；
    加的那一段不带 /，键的换算不折它，各片各一个键；这件的键：父任务给了 RUN_WITH_MEMORY_CAP_KEY 的照它，不然父命令照词折叠（memory_peaks_lib.py 的 command_key 收词表，
    带空格的条目名、路径仍是一个词，不再用空格接起来再切），截到给加的那一段留出长度为止，加的那一段不被键长上限截掉（task_server.py 的 create_child；
    弄坏开关 shard-key-from-joined-command 下照改前用空格接父命令的词）；同一计划的计划那一步与各片另共用峰值，上面「资源记录」）、
    各自挑机器、各自重试；这件（父任务）自己不起，整件的结局等子任务定。
    ① 计划（plan）：服务挑台起这条命令，环境里 TASK_SERVER_SHARD_PHASE=plan、TASK_SERVER_SHARD_PLAN_DIRECTORY=<计划目录>；命令往计划目录写 plan.json：
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
    一片没成（有了结局、不是 succeeded，人手单独取消的不算）：还在排的兄弟片当场记 cancelled 不起（记 cancel_origin=sibling，这几片不算没成）；在跑的照跑到底、结局照收
    （判过的块照写进各自的库），在跑的全部收齐之后父任务才记结局（取最先没成的那一片），不合并（task_server.py 的 drive_parent_phase、cancel_queued_children；
    弄坏开关 shard-siblings-abandoned 下照改前当场记整件的结局，在跑的没人收）。计划写坏记 shard-plan-refused。整件成了之后删 <库标签>-<号>/ 下各片的产物目录（计划目录留着）。
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
    分卡归服务的 GPU_CARDS_BY_PRIORITY 与 CUDA_VISIBLE_DEVICES、名字以 _BREAK 结尾的弄坏开关、指到仓里的编译目录、其余一切名单外的。出路：unset 它，
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
  内存包装、multi-host-run.sh、gpu-quiesce-run.sh、cargo 换成临时目录里的假脚本（照命令里写的 fake-wrapper-exits=、fake-wrapper-exit-peak=、fake-wrapper-scene=、fake-peer-exit=、fake-quiesce= 演结局，每次被起往调用记录里写一行；
  假包装拒起真的 cargo、rustc，只放行假 cargo；假第二台只认 --peer、--memory-cap、--label 与 --（照真的那样，送与取回的参数认不出、退 2），
  起自证的假分片入口或命令里有记号词 fake-peer-run 时照真的那样在这一刻的工作目录里跑（第二台在同一棵共享的树里），编译目录给这一格临时目录旁边的假第二台本地目录下的、跑完删），
  两台的余量与卡换成一份 JSON；各格的假仓根放在假共享根下（仓也在共享根下），里面有一份只有表头的登记表 .claude/gate.d/verdict-library-scripts.tsv；线程包装 capped.sh 是真的。
  尝试的 scope 一律换成假的（TASK_SERVER_FAKE_SCOPE_DIRECTORY；格里不另给的是一个空目录：峰值量不到、回收不掉的用量算 0）：尝试不在自己的单元里时
  （弄坏开关 no-detach）、自证本身套在真内存包装的 scope 里跑时（经 task-server 提交），尝试读到的都不是自证自己那一层的 scope，判定不随起法变。
  尝试单元照旧经真的 systemd-run --user 起（单元名带这一份库的随机标签），跑的只有 bash 与 python 小脚本；等放行文件的那几条至多等 60 秒自己退出。
  不起编译、不起重活、不连第二台、不上卡。收尾停掉这一格的服务（自己起的进程号）与它起过、还在跑的单元（按库里记的单元名逐个停），删临时目录。
  各格并行，并行度取每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads 现读），取不出登记值（登记文件不在、写坏，SINGLEFS_THREAD_CAP 不是正整数）整份判红、不退回按核数，每格的输出与退出码各写自己的文件，按格名次序回读、数份数。
  只跑几格：TASK_SERVER_SELFTEST_CELLS=<格名,…>（单跑一格与它在全部格里跑判定相同；门禁与提交前跑的都不设）。

只在自证里认的变量（TASK_SERVER_SELFTEST_MARK 指到自证建的标记文件、内容是 task-server-selftest 才认；别处设了，服务、客户端与 control 一律拒、退 2、什么都不做）：
  TASK_SERVER_BREAK 弄坏开关；TASK_SERVER_SELFTEST_FAST 定时缩短；TASK_SERVER_SELFTEST_WORLD 假脚本写调用记录的目录（服务另在它下面的 scheduler-reads 记排队读预测与余量的每一次）；
  TASK_SERVER_FAKE_RESOURCES 两台余量与卡的 JSON（写了 recheck 的，起之前复核读的那一台照它；每台的报告里写了 disk_available 的、或 df 里写了那一台（{台: 演 df 的输出}）的
    照它判盘余量，都没写的当盘够；down 里写了那一台（{台: 一句原因}）的演它此刻用不了；memory_caps（{台: 字节}）是那台总上限另收到的数；
    每台报告的 memory 里写着那台此刻的余量 slice_margin（自证喂的定值，缺省 2G；写 null 演量不到），服务照它判；报告里写了 slice_scopes 的
    （{单元名: {unreclaimable, run_label}}，memory_peaks_lib.py 文件头「在跑的 scope」），服务照它算那台在跑的待涨量，没写的那几条记整份上限）；
  TASK_SERVER_MEMORY_WRAPPER、TASK_SERVER_PEER_RUNNER、TASK_SERVER_GPU_QUIESCE、TASK_SERVER_SYSTEMD_RUN、TASK_SERVER_CONTROL_SYSTEMCTL、TASK_SERVER_NVIDIA_SMI 换掉对应的程序；
  TASK_SERVER_FAKE_SCOPE_DIRECTORY 尝试的 scope 换成一份假 cgroup 目录（上面「自证」）；TASK_SERVER_FAKE_REPOSITORY_ROOT 判能不能派第二台时当仓根的目录，也是找编出来的二进制同名源时的仓根（compiled_binary_source）；
  TASK_SERVER_SESSION_BOUNDARY_PID 往上读会话号读到这个进程号为止（不含它）；TASK_SERVER_SELFTEST_MISSING_MODULES 当这几个模块不在；
  TASK_SERVER_SELFTEST_CRASH_POINT=<before-unit|after-unit>:<文件> 那份文件在时服务在写了单元名之后、起单元之前（或起了之后）当场退出；
  TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE=<文件> 那份文件在时 /health 的试写当失败；TASK_SERVER_SELFTEST_ALLOW_TMPFS 状态目录在 tmpfs 上也不拒（临时目录可能在 tmpfs 上）；
  TASK_SERVER_SELFTEST_AGED_SECONDS 提交满这么多秒算高优先；TASK_SERVER_SELFTEST_STALL_SECONDS 日志这么多秒没涨算卡住；TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS 起来后先宽限这么多秒；TASK_SERVER_SELFTEST_PLACEMENT_PAUSED=<文件> 那份文件在时排队线程照判、判能起的不起（记成判能起却没起成）；TASK_SERVER_SELFTEST_PLACEMENT_REFUSED=<文件> 那份文件在时排队线程把每条都判成不能起（演派活判法判错，看进展那一道该报）；
  TASK_SERVER_SELFTEST_SOCKET_ROOT 各格的 socket 目录建在它下面（上面「自证」）；TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS 读假余量每次先睡这么多秒（演余量读得慢）；
  TASK_SERVER_NFS_EXPORTS 当 /var/lib/nfs/etab 读的那份文件、TASK_SERVER_PEER_SSH 连第二台起的 ssh（上面「NFS 共享」：自证里换成假的，各格的多机配置写一行 NFS_SHARED_ROOTS
  指到格里的假共享根）；TASK_SERVER_SELFTEST_PAUSE_POINT=<start-checked|claimed|cancel-unlocked>:<目录> 目录里 arm-<点> 在时服务在那一点建 reached-<点>、等 release-<点>
  （start-checked 是判过 queued、记尝试之前，claimed 是记了尝试、起单元之前，cancel-unlocked 是取消记了 cancel_requested、放了锁、停单元之前；task_server.py 的 selftest_pause）；TASK_SERVER_SELFTEST_STOP_FAILS=<文件> 那份文件在时
  下一次停单元不发 systemctl stop、删掉它（演停失败一次）；TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS 起单元的时限；TASK_SERVER_PRUNE_SERVER_CHECK 命令退 75 时
  当 prune-server/control check 起的那一份可执行文件。TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES 弄坏开关 recorded-cap-floored 下兜到的那个下限（字节；各格的小世界写 0，
  开关只红点名的几格；retry-cap-not-floored、recorded-cap-not-floored、cap-hit-rerun-from-record 三格写空串、照真的 MEMORY_FLOOR_BYTES）。TASK_SERVER_SELFTEST_LONG_RUN_SECONDS「跑得久」的界（秒，换掉 LONG_RUN_SECONDS；上面「续跑」）。
  TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS 跑满两小时打回的界（秒，换掉 task_server.py 的 RUN_LIMIT_SECONDS；上面「错误类型」over-two-hours：自证不真等两小时）；
  TASK_SERVER_SELFTEST_NFS_SSH_SECONDS 连第二台那一次 ssh 的时限（秒，换掉 NFS_SSH_SECONDS）、TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS 超时之后隔多久复查（秒，换掉 NFS_RECHECK_PAUSE_SECONDS；上面「NFS 共享」）。
  TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS 排队过久的头一档（秒，换掉 QUEUE_ATTENTION_MINUTES × 60）；TASK_SERVER_SELFTEST_OVERRUN_ATTENTION_FACTOR 超过记录用时的头一档（倍，
  换掉 OVERRUN_ATTENTION_FACTOR）；TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS 日志长期不涨的头一档（秒，换掉 QUIET_ATTENTION_MINUTES × 60）；
  TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS 卡在 IO 上的头一档（秒，换掉 IO_WAIT_ATTENTION_MINUTES × 60）；
  TASK_SERVER_SELFTEST_GPU_BUSY_QUIET_ATTENTION_SECONDS 卡上有它的计算进程而日志与 CPU 都不涨的头一档（秒，换掉 GPU_BUSY_QUIET_ATTENTION_MINUTES × 60）；TASK_SERVER_SELFTEST_PROC_ROOT 本机读进程状态与父进程号的目录
  换成它（当 /proc 读；task_server.py 的 scope_blocked_processes：假 scope 的 cgroup.procs 里点名的进程号在它下面有一份假的 stat，演 D 状态）；TASK_SERVER_SELFTEST_ADVANCE_FAILS=<文件> 那份文件在时每一轮推分片父任务都抛一个错（演分片父任务推不动；task_server.py 的 selftest_advance_fault）。
  TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS 交任务时会话名下结局满这么多秒没人取就打严厉警告、结局没人取再推的头一档（换掉 UNCLAIMED_WARNING_MINUTES × 60）；
  TASK_SERVER_CONTROL_SYSTEMCTL 另换服务盯自己的单元在不在时起的 systemctl（task_server.py 的 own_unit_state）。
  TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS 跑得久而核没用起来的界（秒，换掉 task_server.py 的 LONG_RUN_ATTENTION_SECONDS）。
  TASK_SERVER_SELFTEST_SESSIONS_DIRECTORY 服务按会话号找收件 socket 时读的会话登记目录（换掉 ~/.claude/sessions；task_server.py 文件头 ⑥「收件地址」）；
  TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS 交了没确认满这么多秒服务取消（换掉 task_server.py 的 CONFIRM_TIMEOUT_SECONDS）；
  TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS 不随时长涨的提醒同一档推过之后再推的头一档（秒，换掉 task_server.py 的 ANOMALY_REPUSH_SECONDS）。
  TASK_SERVER_SELFTEST_CRASH_POINT 的点另有 attempt-claimed（起尝试的事务里、插了尝试、改任务之前）与分片推进的
  parent-inserted、child-finished-<plan|shard|merge>、shard-inserted、merge-outputs-written、merge-inserted。

弄坏开关 TASK_SERVER_BREAK=<项>（逗号连几个；只给 --selftest 证红，认不出的项自证退 2）。读开关只在这一份的 broken。打开后自证红在哪一格
（失败行「✗ task_server_lib.py 自证：<格名>：…」；末尾「✗ task_server_lib.py 自证 N 格里 M 格不对」是汇总）：
  TASK_SERVER_BREAK=fallback-run-when-unreachable  客户端 submit 连不上服务时退回直接执行命令（task 的 submit）
      「✗ task_server_lib.py 自证：server-unreachable：」
  TASK_SERVER_BREAK=exit-code-dropped  客户端 wait 一律退 0（task 的 wait）
      「✗ task_server_lib.py 自证：exit-code-passthrough：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、
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
      「✗ task_server_lib.py 自证：cancel-own-session-only：」、「✗ task_server_lib.py 自证：stalled-attempt-stopped：」、「✗ task_server_lib.py 自证：overrun-quiet-log-stopped：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、
      「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：item-estimate-cancelled：」、「✗ task_server_lib.py 自证：cancel-start-race：」、
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：start-timeout-read-back：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」、「✗ task_server_lib.py 自证：cancel-after-natural-end：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」（这两格推的，没复跑：同 stalled-at-cap-retried，停卡住那一条走同一段停法）
      「✗ task_server_lib.py 自证：io-wait-peer-not-stalled：」（推的，没复跑：同 peer-stall-judged-by-peer-cpu，没有 D 状态与读不到的那两条照停，走同一段停法）
      「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」（推的，没复跑：同 stalled-attempt-stopped，卡上有几轮没负载的与没分到卡的那两条照停，走同一段停法）
      「✗ task_server_lib.py 自证：card-zero-utilization-stalled：」（推的，没复跑：利用率是 0 的那一条走同一段停法）
      「✗ task_server_lib.py 自证：cancelled-before-peer-trip-built：」（单跑这一格复跑过：任务 8993 红在这一格；取消不停单元，回执停在 not-started 的那一条照样没停下来）
  TASK_SERVER_BREAK=queued-cancel-left-queued  取消排着的任务只记要取消、不当场收尾，任务留在队里（task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：cancel-queued：」
      「✗ task_server_lib.py 自证：cancel-start-race：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=finished-cancel-overwrites  取消已有结局的任务时把结局改记 cancelled（task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：cancel-finished：」
  TASK_SERVER_BREAK=sharded-cancel-children-left  取消分片的父任务只给父任务记 cancelled，排着的子任务不收尾、在跑的不停（task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：cancel-sharded-in-plan：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
  TASK_SERVER_BREAK=signature-unchecked  服务读回不核校验列（task_server.py 的 Store.verified）
      「✗ task_server_lib.py 自证：tampered-row-not-started：」
      「✗ task_server_lib.py 自证：association-column-verified：」、「✗ task_server_lib.py 自证：child-rows-verified：」
  TASK_SERVER_BREAK=tamper-unalerted  核出改过的行不写告警（task_server.py 的 Scheduler.tampered）
      「✗ task_server_lib.py 自证：tampered-row-not-started：」
  TASK_SERVER_BREAK=operations-unlogged  增加与取消不写操作日志（task_server.py 的 Scheduler.operation）
      「✗ task_server_lib.py 自证：operation-log：」、「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」
      「✗ task_server_lib.py 自证：clean-failure-explained：」
  TASK_SERVER_BREAK=unlisted-exit-swallowed  classify_attempt 把表外的退出码记成 succeeded
      「✗ task_server_lib.py 自证：exit-code-passthrough：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：failed-run-estimated：」、
      「✗ task_server_lib.py 自证：unfinished-peak-ignored：」、「✗ task_server_lib.py 自证：shard-retry-isolated：」、「✗ task_server_lib.py 自证：shard-plan-failed：」、「✗ task_server_lib.py 自证：shard-merge-failed：」、
      「✗ task_server_lib.py 自证：long-run-failure-noted：」、「✗ task_server_lib.py 自证：failed-peak-floor：」、「✗ task_server_lib.py 自证：item-estimate-failure-queues-parent：」、「✗ task_server_lib.py 自证：reason-line-per-type：」、
      「✗ task_server_lib.py 自证：peer-command-exit-split：」、「✗ task_server_lib.py 自证：failure-scene-kept-until-collected：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」
  TASK_SERVER_BREAK=card-fault-unread  classify_attempt 不读日志里的 result=card-fault，清场退 78 一律记 run-condition-unmet
      「✗ task_server_lib.py 自证：attention-card-replaced：」、「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：card-fault-replaced-on-another-card：」、
      「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」、「✗ task_server_lib.py 自证：card-fault-unlisted-quarantines-none：」
  TASK_SERVER_BREAK=card-fault-unalerted  记了 card-fault 不写告警（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：error-types：」
      「✗ task_server_lib.py 自证：card-fault-replaced-on-another-card：」、「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」、「✗ task_server_lib.py 自证：card-fault-unlisted-quarantines-none：」
  TASK_SERVER_BREAK=peer-exit-unsplit  classify_attempt 不分第二台，multi-host-run.sh 的 255 记成 command-failed
      「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」
  TASK_SERVER_BREAK=lost-not-recorded  对不回来的尝试一直当在跑，不记 launcher-lost（task_server.py 的 reconcile）
      「✗ task_server_lib.py 自证：error-types：」
  TASK_SERVER_BREAK=cap-growth-skipped  retry_decision 撞了上限照原上限重排（不乘 1.5）
      「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：unrecorded-cap-growth：」、
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」
      「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：ended-attempt-settled-after-restart：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」
      （busy-cpu-pinned-at-cap-retried、peer-pinned-at-cap-busy-cpu-retried 这两格不红，量的：重排那一次的上限由 history_raised_prediction 照同一把键近几次最高乘 1.5 抬到同一档，
      撞上限那一次记的就是 1536M，不靠 retry_decision 乘；别把这两格点进来）
  TASK_SERVER_BREAK=cap-unclamped  retry_decision 乘了 1.5 到或超过那台的总上限也照重排、不打回
      「✗ task_server_lib.py 自证：memory-cap-at-maximum：」
      「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」
  TASK_SERVER_BREAK=host-switch-skipped  retry_decision 在总上限上还撞时不换总上限更大的另一台，直接记 memory-cap-at-maximum
      「✗ task_server_lib.py 自证：memory-cap-at-maximum：」
      「✗ task_server_lib.py 自证：pinned-peer-unavailable：」
  TASK_SERVER_BREAK=retry-overwrites-attempts  重试那一次起的时候把这个任务前面几次尝试的记录删掉，只留这一次（覆盖前一次；重试次数另记在 outcome_counts，不受它影响）
      「✗ task_server_lib.py 自证：build-family-cap-hit-raises：」、「✗ task_server_lib.py 自证：attention-card-replaced：」、「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-growth：」、
      「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：unrecorded-cap-growth：」、「✗ task_server_lib.py 自证：retry-kinds：」、「✗ task_server_lib.py 自证：retry-keeps-earlier-attempts：」、
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」、「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：task-log：」、「✗ task_server_lib.py 自证：ended-attempt-settled-after-restart：」、「✗ task_server_lib.py 自证：card-fault-replaced-on-another-card：」、
      「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」、「✗ task_server_lib.py 自证：workspace-writer-not-retried：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」、
      「✗ task_server_lib.py 自证：shard-siblings-share-peaks：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」（这两格推的，没复跑：前一次尝试的记录被删）
  TASK_SERVER_BREAK=squeeze-retry-skipped  retry_decision 被总上限挤掉（254）不重排
      「✗ task_server_lib.py 自证：retry-kinds：」、「✗ task_server_lib.py 自证：retry-keeps-earlier-attempts：」、「✗ task_server_lib.py 自证：task-log：」、「✗ task_server_lib.py 自证：pause-lets-running-finish：」、「✗ task_server_lib.py 自证：pause-allows-one-session：」
  TASK_SERVER_BREAK=gpu-short-retry-skipped  retry_decision 显存不够（245）不再试
      「✗ task_server_lib.py 自证：retry-kinds：」
  TASK_SERVER_BREAK=server-pending-ignored  服务判放不放得下时不减自己账上在跑、还没涨到量的那一截（task_server.py 的 server_pending）：
      在跑的任务进判放不放得下只经这一处，凡是要“后来的被在跑的占着、排着”“按在跑的记账均衡挑台”的格都连带红
      「✗ task_server_lib.py 自证：queue-holds-unplaceable：」、「✗ task_server_lib.py 自证：pass-booking-carried：」、「✗ task_server_lib.py 自证：unrecorded-burst-balanced：」、「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」、
      「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：shard-siblings-share-peaks：」
  TASK_SERVER_BREAK=launch-when-unplaceable  不要卡的任务此刻放不下（连同哪台都放不下）也照起，起在本机（task_server.py 的 place）：
      “放不下就排着”只在 place 这一处判，凡是要后来的排着、按先后起、等某一样腾出来的格都连带红
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：queue-holds-unplaceable：」、「✗ task_server_lib.py 自证：prediction-computed-once：」、
      「✗ task_server_lib.py 自证：pass-reads-resources-once：」、「✗ task_server_lib.py 自证：pass-booking-carried：」、「✗ task_server_lib.py 自证：launch-rechecked：」、「✗ task_server_lib.py 自证：backfill-when-head-blocked：」、
      「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」、「✗ task_server_lib.py 自证：simulated-two-host-workload：」、
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：admission-booked-at-cap：」、
      「✗ task_server_lib.py 自证：whole-and-fragment-alternate：」、「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、「✗ task_server_lib.py 自证：aged-tasks-do-not-hold-fitting-tasks：」、「✗ task_server_lib.py 自证：cpu-gate-holds-busy-host：」、
      「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、
      「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：item-estimate-summed-from-rates：」、「✗ task_server_lib.py 自证：item-estimate-failure-queues-parent：」、
      「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」、「✗ task_server_lib.py 自证：shard-plan-and-merge-estimated-by-command：」、「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、
      「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、「✗ task_server_lib.py 自证：reason-line-per-type：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」、「✗ task_server_lib.py 自证：shard-siblings-share-peaks：」、
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」
  TASK_SERVER_BREAK=queue-reversed  从后到的往先到的取（task_server.py 的 place_queued）
      「✗ task_server_lib.py 自证：arrival-order：」、「✗ task_server_lib.py 自证：pass-booking-carried：」、「✗ task_server_lib.py 自证：whole-and-fragment-alternate：」、「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、
      「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、
      「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、「✗ task_server_lib.py 自证：failed-start-not-counted：」
  TASK_SERVER_BREAK=backfill-disabled  排头放不下时后面放得下的也不起（退回严格排队；task_server.py 的 place_queued）
      「✗ task_server_lib.py 自证：card-whole-seals-while-card-busy：」、「✗ task_server_lib.py 自证：backfill-when-head-blocked：」、「✗ task_server_lib.py 自证：simulated-two-host-workload：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：whole-and-fragment-alternate：」、
      「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、「✗ task_server_lib.py 自证：aged-tasks-do-not-hold-fitting-tasks：」、「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、
      「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」
      「✗ task_server_lib.py 自证：aged-whole-keeps-group-sealed：」（红是量的；原因读代码推的：整片放不下时 behind 把其后碎片的排队原因改写成「排在任务 N 后面」，不写封组）
  TASK_SERVER_BREAK=inner-commands-unchecked  bash / sh -c 引号里的命令不照提交规则判（task_server.py 的 nested_problem）
      「✗ task_server_lib.py 自证：nested-dispatch-refused：」
      「✗ task_server_lib.py 自证：retired-variables-refused：」
  TASK_SERVER_BREAK=retry-pinned-to-host  撞上限重排钉在同一台（task_server_lib.py 的 retry_decision）
      「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」
  TASK_SERVER_BREAK=host-specific-variables-sent-to-peer  派第二台时把预编库这类按台取值的、值是本机绝对路径而不在共享根下的变量也带过去（task_server.py 的 attempt_command）
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」
  TASK_SERVER_BREAK=entry-quiesce-skipped  按名字认出要卡的入口照改前当成自己清场、服务不给它套清场（task_server.py 的 attempt_command、NAMED_GPU_REASON）
      「✗ task_server_lib.py 自证：gpu-entry-scripts-recognized：」
  TASK_SERVER_BREAK=mutation-gate-gpu-unread  变异门禁 crates-mutation-replay.sh 不按变异表里有没有上卡的行认成要卡、不由服务套清场（task_server.py 的 mutation_gate_uses_gpu）
      「✗ task_server_lib.py 自证：gpu-entry-scripts-recognized：」
  TASK_SERVER_BREAK=cpu-ignored-for-stall  日志满 STALL_SECONDS 没涨就停，不看 CPU 在不在涨（task_server.py 的 check_stall）
      「✗ task_server_lib.py 自证：computing-quietly-not-stalled：」
      「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」
      「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」、「✗ task_server_lib.py 自证：attention-quiet-busy-log：」（这两格推的，没复跑：CPU 在涨也照停，wait 等到的是结局不是提醒）
      「✗ task_server_lib.py 自证：card-idle-cpu-computing-not-stalled：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=prove-red-gpu-unread  prove-red.sh 不读它点名的变异行，带 gpu 特性的也认成不要卡（task_server.py 的 prove_red_uses_gpu）
      「✗ task_server_lib.py 自证：gpu-entry-scripts-recognized：」
  TASK_SERVER_BREAK=queued-not-revalidated  服务起来时不再核还在排的任务（task_server.py 的 revalidate_queued）
      「✗ task_server_lib.py 自证：queued-revalidated-on-restart：」
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
  TASK_SERVER_BREAK=overrun-not-stopped  已跑超过记录用时 OVERRUN_STOP_FACTOR 倍、日志满 STALL_SECONDS 没涨，CPU 在涨照旧不停（task_server.py 的 check_stall）
      「✗ task_server_lib.py 自证：overrun-quiet-log-stopped：」
  TASK_SERVER_BREAK=stall-ignored  日志满 STALL_SECONDS 没涨也不停（task_server.py 的 check_stall）
      「✗ task_server_lib.py 自证：stalled-attempt-stopped：」、「✗ task_server_lib.py 自证：overrun-quiet-log-stopped：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、
      「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」、「✗ task_server_lib.py 自证：attention-quiet-busy-log：」（这三格推的，没复跑：不停就一直跑到命令自己退；
      日志多久没涨照判卡住记的那一份，开关下不记，提醒也报不出）
      「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」、「✗ task_server_lib.py 自证：io-wait-peer-not-stalled：」（这两格推的，没复跑：D 状态只在判卡住那一段里读，
      开关下不读、io-wait 提醒报不出；没有 D 状态与读不到的那两条不停）
      「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」（推的，没复跑：卡上的负载只在判卡住那一段里记，开关下不记、gpu-busy-quiet 报不出；卡上有几轮没负载的与没分到卡的那两条不停）
      「✗ task_server_lib.py 自证：card-zero-utilization-stalled：」（推的，没复跑：利用率是 0 的那一条不停、等满安静时限）
      「✗ task_server_lib.py 自证：card-idle-polling-stalled：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=running-progress-unreported  在跑的那一次不报已跑多久、日志多久没涨、在用核数（task_server.py 的 summary）
      「✗ task_server_lib.py 自证：running-status-shows-progress：」、「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」、「✗ task_server_lib.py 自证：suspected-stall-notified：」
  TASK_SERVER_BREAK=wrapper-prefixes-allowed  命令里套 nice、timeout、capped.sh、two-logs.sh、gpu-quiesce-run.sh 照收（task_server_lib.py 的 nested_dispatch）
      「✗ task_server_lib.py 自证：nested-dispatch-refused：」
  TASK_SERVER_BREAK=resource-arguments-allowed  命令里写 --memory、--threads、--peer 这类参数照收（task_server_lib.py 的 nested_dispatch）
      「✗ task_server_lib.py 自证：queued-revalidated-on-restart：」、「✗ task_server_lib.py 自证：nested-dispatch-refused：」

  TASK_SERVER_BREAK=estimate-only-succeeded  预计用时只认退 0 的那几次，命令自己退非 0 跑到退出的不算（task_server.py 的 command_estimate）
      「✗ task_server_lib.py 自证：failed-run-estimated：」
  TASK_SERVER_BREAK=waiting-reason-unreported  排着的任务不报在等什么（task_server.py 的 summary 与 session_tasks）
      「✗ task_server_lib.py 自证：pause-allow-waiting-text：」、「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：launch-rechecked：」、「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、
      「✗ task_server_lib.py 自证：queued-shows-waiting-reason：」、「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、「✗ task_server_lib.py 自证：cpu-gate-holds-busy-host：」、
      「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、
      「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、「✗ task_server_lib.py 自证：build-disk-predicted：」
  TASK_SERVER_BREAK=nofile-limit-dropped  起尝试单元不带 LimitNOFILE，软上限落回用户级 systemd 的 1024（task_server.py 的 start_unit）
      「✗ task_server_lib.py 自证：attempt-unit-nofile-raised：」
  TASK_SERVER_BREAK=gpu-entry-scripts-unrecognized  不认脚本里面才清场用卡的入口（task_server.py 的 GPU_ENTRY_SCRIPT_NAMES）
      「✗ task_server_lib.py 自证：gpu-entry-scripts-recognized：」
  TASK_SERVER_BREAK=gpu-unrecognized  不认 GPU 的命令（task_server.py 的 gpu_need）
      「✗ task_server_lib.py 自证：attention-card-replaced：」、「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：gpu-highest-free-card：」、
      「✗ task_server_lib.py 自证：gpu-entry-scripts-recognized：」、「✗ task_server_lib.py 自证：gpu-without-cards-unmet：」、「✗ task_server_lib.py 自证：gpu-runs-only-when-code-runs：」、「✗ task_server_lib.py 自证：card-labels-by-address：」、
      「✗ task_server_lib.py 自证：suspected-stall-card-busy-not-reported：」、「✗ task_server_lib.py 自证：python-entry-gpu-recognized：」、「✗ task_server_lib.py 自证：gpu-argument-declaration-recognized：」、「✗ task_server_lib.py 自证：card-fault-replaced-on-another-card：」、
      「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」、「✗ task_server_lib.py 自证：card-fault-unlisted-quarantines-none：」、「✗ task_server_lib.py 自证：temporary-service-heavy-refused：」、「✗ task_server_lib.py 自证：gpu-binary-declaration-recognized：」、
      「✗ task_server_lib.py 自证：gpu-two-cards-same-host：」、「✗ task_server_lib.py 自证：gpu-three-cards-only-where-enough：」、「✗ task_server_lib.py 自证：gpu-cards-short-never-fits：」
      「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」（推的，没复跑：不认要卡就不读卡，卡上一直有负载的那一条也照停）
      「✗ task_server_lib.py 自证：card-zero-utilization-stalled：」（推的，没复跑：不认要卡就不读卡，利用率 37% 的那一条也照停）
  TASK_SERVER_BREAK=cardless-gpu-launched  GPU 任务而多机配置没点名能用的卡时不判 run-condition-unmet，照不要卡的任务起（task_server.py 的 place）
      「✗ task_server_lib.py 自证：gpu-without-cards-unmet：」
      「✗ task_server_lib.py 自证：card-labels-by-address：」
  TASK_SERVER_BREAK=card-priority-ignored  GPU 任务从优先级最低的卡往前挑
      「✗ task_server_lib.py 自证：attention-card-replaced：」、「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：gpu-highest-free-card：」、「✗ task_server_lib.py 自证：card-fault-replaced-on-another-card：」、
      「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」、「✗ task_server_lib.py 自证：card-fault-unlisted-quarantines-none：」、「✗ task_server_lib.py 自证：gpu-two-cards-same-host：」、「✗ task_server_lib.py 自证：gpu-three-cards-only-where-enough：」
  TASK_SERVER_BREAK=card-labels-all-local  卡的台标签不照 HOSTS 认地址，一律当服务所在这一台的卡（task_server.py 的 parse_cards）
      「✗ task_server_lib.py 自证：card-labels-by-address：」
      「✗ task_server_lib.py 自证：gpu-two-cards-same-host：」、「✗ task_server_lib.py 自证：gpu-three-cards-only-where-enough：」、「✗ task_server_lib.py 自证：gpu-cards-short-never-fits：」（推的，没复跑）
  TASK_SERVER_BREAK=card-busy-ignored  GPU 任务挑卡不看那张卡上有没有服务起的任务在跑
      「✗ task_server_lib.py 自证：gpu-highest-free-card：」
  TASK_SERVER_BREAK=card-wait-blocks-all  GPU 任务等空卡时，后面不要卡的任务也不起
      「✗ task_server_lib.py 自证：gpu-highest-free-card：」
  TASK_SERVER_BREAK=no-detach  尝试不起成脱离服务的单元，起成服务自己的子进程，服务停下时跟着停（task_server.py 的 start_unit）
      「✗ task_server_lib.py 自证：restart-keeps-running：」、「✗ task_server_lib.py 自证：unit-start-failed：」、「✗ task_server_lib.py 自证：cancel-own-session-only：」、「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」、
      「✗ task_server_lib.py 自证：leftover-processes-stopped：」、「✗ task_server_lib.py 自证：aged-tasks-do-not-hold-fitting-tasks：」、「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：association-column-verified：」、
      「✗ task_server_lib.py 自证：start-timeout-read-back：」、「✗ task_server_lib.py 自证：failed-start-not-counted：」
  TASK_SERVER_BREAK=start-failure-ignored  systemd-run 退非 0 也当起成了（task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：unit-start-failed：」
      「✗ task_server_lib.py 自证：failed-start-not-counted：」
  TASK_SERVER_BREAK=start-twice-after-restart  重启之后不看单元在不在、起跑标记有没有，「起单元之前」的那一次一律当没起、再起一份（task_server.py 的 reconcile）
      「✗ task_server_lib.py 自证：start-once：」
  TASK_SERVER_BREAK=resources-unrecorded  尝试结束时不把用时之外的资源（内存峰值、核数、显存峰值）写进库（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：resource-record：」、「✗ task_server_lib.py 自证：unmeasured-not-zero：」、「✗ task_server_lib.py 自证：unfinished-peak-ignored：」、「✗ task_server_lib.py 自证：completed-after-cap-hit：」、
      「✗ task_server_lib.py 自证：failed-peak-floor：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、
      「✗ task_server_lib.py 自证：short-command-peak-recorded：」、「✗ task_server_lib.py 自证：build-family-peak-shared：」、「✗ task_server_lib.py 自证：build-family-declared：」、
      「✗ task_server_lib.py 自证：build-family-from-arguments：」（后一格推的，没复跑：同族那一条的峰值记不下）
  TASK_SERVER_BREAK=record-ignored-for-cap  定上限不看服务自己的资源记录，只照没记录的打底（task_server.py 的 prediction_of）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：cap-hit-rerun-from-record：」、「✗ task_server_lib.py 自证：backfill-when-head-blocked：」、
      「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：resource-record：」、
      「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」、「✗ task_server_lib.py 自证：unmeasured-not-zero：」、「✗ task_server_lib.py 自证：simulated-two-host-workload：」、
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：aged-head-at-slice-total-starts：」、「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：admission-booked-at-cap：」、「✗ task_server_lib.py 自证：whole-and-fragment-alternate：」、「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」、
      「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、「✗ task_server_lib.py 自证：aged-tasks-do-not-hold-fitting-tasks：」、「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：failed-peak-floor：」、
      「✗ task_server_lib.py 自证：retry-cap-not-floored：」、「✗ task_server_lib.py 自证：recorded-cap-not-floored：」、「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、
      「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、
      「✗ task_server_lib.py 自证：item-estimate-summed-from-rates：」、「✗ task_server_lib.py 自证：item-estimate-failure-queues-parent：」、「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」、「✗ task_server_lib.py 自证：shard-plan-and-merge-estimated-by-command：」、
      「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、
      「✗ task_server_lib.py 自证：reason-line-per-type：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」、「✗ task_server_lib.py 自证：aged-whole-keeps-group-sealed：」（这三格量的）、
      「✗ task_server_lib.py 自证：card-whole-seals-while-card-busy：」（推的，没复跑：同样没看记录，整片只记 4G、不算整片、不封组，六条碎片都起）
      「✗ task_server_lib.py 自证：workspace-writer-initial-cap-headroom：」、「✗ task_server_lib.py 自证：both-hosts-sealed-small-unestimated-starts：」、
      「✗ task_server_lib.py 自证：fragment-group-drops-finished：」（推的，没复跑：记下的峰值都不看，上限都是 4G、整片不算整片）
      （peer-pinned-at-cap-busy-cpu-retried 这一格不红，量的：没看记录头一次的上限 4096M，第二台报的此刻内存 15G 照样不低于它的 98%，停掉重排跑完）
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
  TASK_SERVER_BREAK=retry-build-directory-kept  重排换了台，上一次在本机留着的那一份编译目录不删（task_server.py 的 start_attempt 换台那一支）
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」
  TASK_SERVER_BREAK=clean-removes-nothing  task clean 不删编译目录（task_server.py 的 clean）
      「✗ task_server_lib.py 自证：build-directory-outside-repository：」、「✗ task_server_lib.py 自证：clean-failure-explained：」
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
      「✗ task_server_lib.py 自证：health-head-cpu-gate-not-starved：」、「✗ task_server_lib.py 自证：health-head-cards-short-not-starved：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、
      「✗ task_server_lib.py 自证：progress-stall-reported：」
  TASK_SERVER_BREAK=health-head-judged-apart  /health 照改前另判一套排头放不放得下（只看内存与有没有一张空卡），不读派活给排头的结论（task_server.py 的 head_judged_apart；
      任务 7568、7615、2365 那样被误报）
      「✗ task_server_lib.py 自证：health-head-cpu-gate-not-starved：」、「✗ task_server_lib.py 自证：health-head-cards-short-not-starved：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、
      「✗ task_server_lib.py 自证：progress-stall-reported：」
  TASK_SERVER_BREAK=progress-unwatched  不看进展：某台照公式放得下排着的最小那条却一直一条都没起，/health 照回 ok（task_server.py 的 progress_problem）
      「✗ task_server_lib.py 自证：progress-stall-reported：」
  TASK_SERVER_BREAK=progress-gates-ignored  看进展照公式放得下就算，不看卡、CPU 闸、编译目录占盘与封着组等腾空（task_server.py 的 progress_smallest_fitting）
      「✗ task_server_lib.py 自证：health-head-cpu-gate-not-starved：」、「✗ task_server_lib.py 自证：health-head-cards-short-not-starved：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」
  TASK_SERVER_BREAK=card-wait-blocks-later-cards  前面一条要卡的在等卡，后面要卡的照改前全都不起，不看它们落的是不是那几台的卡（task_server.py 的 place_queued_into；任务 7615、7618）
      「✗ task_server_lib.py 自证：card-wait-holds-only-its-host：」
  TASK_SERVER_BREAK=wait-notify-skipped  记下结局时不叫醒在等的 wait（task_server.py 的 notify_finished），wait 只能等兜底那一轮
      「✗ task_server_lib.py 自证：thirty-second-tick-prompt：」
  TASK_SERVER_BREAK=in-repository-target-accepted  调用方给的编译目录指到仓里也收（target_directory_refusal，客户端与服务都用它）
      「✗ task_server_lib.py 自证：build-directory-outside-repository：」
  TASK_SERVER_BREAK=unmeasured-recorded-as-zero  量不到峰值的那几次当成 0 记进记录、占近几次的名额（task_server.py 的 finish_attempt 与 recorded_prediction）
      「✗ task_server_lib.py 自证：unmeasured-not-zero：」、「✗ task_server_lib.py 自证：short-command-peak-recorded：」（两样都没有的那一条记成 0）
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」
  TASK_SERVER_BREAK=estimate-unreported  排队时不报预计用时（task_server.py 的 estimate_of）
      另有七格读预计用时（estimated_seconds 或 estimate_of）：按项估的 item-estimate-summed-from-rates，status 报记录用时的 running-status-shows-progress，
      超跑停的 overrun-quiet-log-stopped，封组插空看时长的 sealed-gap-takes-short-task、sealed-gap-holds-longer-task、unestimated-blocker-counted-by-elapsed、gap-fill-room-exhausted-checks-length，
      每格一个子进程、各有各的服务与库，没有格靠别的格留下的状态红：
      「✗ task_server_lib.py 自证：attention-overrun：」、「✗ task_server_lib.py 自证：queued-estimate：」、「✗ task_server_lib.py 自证：running-status-shows-progress：」、「✗ task_server_lib.py 自证：overrun-quiet-log-stopped：」、
      「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、
      「✗ task_server_lib.py 自证：item-estimate-summed-from-rates：」、「✗ task_server_lib.py 自证：estimate-by-folded-command-key：」、「✗ task_server_lib.py 自证：estimate-basis-command-sentence：」、「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」、
      「✗ task_server_lib.py 自证：shard-plan-and-merge-estimated-by-command：」、「✗ task_server_lib.py 自证：shard-pieces-estimated-by-own-history：」
  TASK_SERVER_BREAK=balance-ignored  两台都放得下时取排在最前的那台，不比剩余占比（task_server.py 的 place）
      「✗ task_server_lib.py 自证：balanced-when-both-fit：」、「✗ task_server_lib.py 自证：unrecorded-burst-balanced：」
      「✗ task_server_lib.py 自证：simulated-two-host-workload：」、「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」
  TASK_SERVER_BREAK=unrecorded-default-grown  没有记录的命令上限与记账都取 UNRECORDED_MEMORY_DEFAULT_BYTES 乘 CAP_GROWTH_FACTOR（6144M；task_server.py 的 unrecorded_prediction）
      「✗ task_server_lib.py 自证：unrecorded-default-small：」、「✗ task_server_lib.py 自证：unrecorded-cap-growth：」、「✗ task_server_lib.py 自证：unfinished-peak-ignored：」、「✗ task_server_lib.py 自证：failed-peak-floor：」、
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」、「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、
      「✗ task_server_lib.py 自证：shard-siblings-share-peaks：」、「✗ task_server_lib.py 自证：build-family-peak-shared：」、「✗ task_server_lib.py 自证：build-family-declared：」
      「✗ task_server_lib.py 自证：history-peak-unrecorded-default：」、「✗ task_server_lib.py 自证：history-peak-retry-jumps：」（后两格推的，没复跑）、
      「✗ task_server_lib.py 自证：build-family-from-arguments：」（推的，没复跑：参数缺了的那一条照 6144M 起）、「✗ task_server_lib.py 自证：build-family-cap-hit-raises：」（推的，没复跑：另一族的一条第 1 次照 6144M 起、不是 4096M）
  TASK_SERVER_BREAK=unrecorded-booked-above-cap  没有记录的命令上限取小缺省、记账取小缺省乘 CAP_GROWTH_FACTOR（6144M；task_server.py 的 unrecorded_prediction）
      「✗ task_server_lib.py 自证：unrecorded-default-small：」
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」、「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」
  TASK_SERVER_BREAK=unfinished-peaks-counted  服务自己的尝试不分结局，量到的峰值都进定额度（改前的样子；task_server.py 的 recorded_prediction）
      「✗ task_server_lib.py 自证：unfinished-peak-ignored：」
      「✗ task_server_lib.py 自证：failed-peak-floor：」
  TASK_SERVER_BREAK=cap-hit-not-counted  撞了这一条上限的尝试不算跑完、它的上限不进定额度（task_server.py 的 recorded_prediction）
      「✗ task_server_lib.py 自证：cap-hit-rerun-from-record：」、「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」
      「✗ task_server_lib.py 自证：history-peak-window-raises：」、「✗ task_server_lib.py 自证：history-peak-retry-jumps：」（后两格推的，没复跑）
  TASK_SERVER_BREAK=completed-peak-ignored  退 0 的尝试量到的峰值不进定额度（task_server.py 的 recorded_prediction）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：cap-hit-rerun-from-record：」、「✗ task_server_lib.py 自证：backfill-when-head-blocked：」、
      「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：resource-record：」、
      「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」、「✗ task_server_lib.py 自证：unmeasured-not-zero：」、「✗ task_server_lib.py 自证：simulated-two-host-workload：」、
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：aged-head-at-slice-total-starts：」、「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：admission-booked-at-cap：」、「✗ task_server_lib.py 自证：whole-and-fragment-alternate：」、「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」、
      「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、「✗ task_server_lib.py 自证：aged-tasks-do-not-hold-fitting-tasks：」、「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」、
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」、「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、
      「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：item-estimate-summed-from-rates：」、
      「✗ task_server_lib.py 自证：item-estimate-failure-queues-parent：」、「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」、「✗ task_server_lib.py 自证：shard-plan-and-merge-estimated-by-command：」、「✗ task_server_lib.py 自证：peer-booked-like-local：」、
      「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、「✗ task_server_lib.py 自证：reason-line-per-type：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、
      「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」、「✗ task_server_lib.py 自证：aged-whole-keeps-group-sealed：」（这三格量的）、
      「✗ task_server_lib.py 自证：card-whole-seals-while-card-busy：」（推的，没复跑：跑成的峰值记不下，整片只记 4G、不算整片、不封组，六条碎片都起）
      「✗ task_server_lib.py 自证：workspace-writer-initial-cap-headroom：」、「✗ task_server_lib.py 自证：both-hosts-sealed-small-unestimated-starts：」、
      「✗ task_server_lib.py 自证：fragment-group-drops-finished：」（推的，没复跑：记下的峰值都不进，上限都是 4G、整片不算整片）
      （peer-pinned-at-cap-busy-cpu-retried 这一格不红，量的：记不下峰值时头一次的上限 4096M，第二台报的此刻内存 15G 照样不低于它的 98%，停掉重排跑完）
  TASK_SERVER_BREAK=peer-overflow-blocked  本机在前的任务本机此刻放不下（还不到永远放不下）时不派第二台，留在队里等本机（task_server.py 的 place）
      「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」、「✗ task_server_lib.py 自证：every-command-reaches-peer：」、
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：simulated-two-host-workload：」、「✗ task_server_lib.py 自证：peer-when-local-full：」、
      「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」、「✗ task_server_lib.py 自证：source-repository-passed-to-command：」、「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」、「✗ task_server_lib.py 自证：peer-admission-marked：」、
      「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：pinned-cap-by-host：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」、「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」、
      「✗ task_server_lib.py 自证：verdict-library-family-refused：」、「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」
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
      「✗ task_server_lib.py 自证：pause-allows-one-session：」、「✗ task_server_lib.py 自证：pause-holds-new-attempts：」、「✗ task_server_lib.py 自证：attention-shard-advance-failed：」、「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、
      「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：shard-retry-isolated：」、「✗ task_server_lib.py 自证：cancel-sharded-in-plan：」、
      「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：shard-plan-failed：」、「✗ task_server_lib.py 自证：shard-merge-failed：」、「✗ task_server_lib.py 自证：unsharded-arguments：」、
      「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：shard-plan-and-merge-estimated-by-command：」、「✗ task_server_lib.py 自证：shard-pieces-estimated-by-own-history：」、「✗ task_server_lib.py 自证：sharded-progress-recovered：」、
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：association-column-verified：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」、「✗ task_server_lib.py 自证：child-rows-verified：」、「✗ task_server_lib.py 自证：shard-siblings-share-peaks：」、
      「✗ task_server_lib.py 自证：shard-child-key-folded-by-words：」
  TASK_SERVER_BREAK=unsharded-arguments-sharded  第一个参数是 --selftest、--list-items、--help、-h（或参数里有 --selftest）的也照分片（first_argument_matches）
      「✗ task_server_lib.py 自证：unsharded-arguments：」
  TASK_SERVER_BREAK=shard-retry-restarts-all  一片重排时把已经成了的兄弟片也重排、再跑一遍（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：shard-retry-isolated：」
  TASK_SERVER_BREAK=merge-before-all-green  有一片成了就起合并，不等别的片（task_server.py 的 drive_parent）
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：shard-retry-isolated：」、
      「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：shard-pieces-estimated-by-own-history：」、「✗ task_server_lib.py 自证：sharded-progress-recovered：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、
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
      「✗ task_server_lib.py 自证：launch-rechecked：」、「✗ task_server_lib.py 自证：shard-siblings-share-peaks：」（后一格推的，没复跑：排着的片照排队那一轮存的 4096M 起）
  TASK_SERVER_BREAK=stalled-at-cap-not-retried  判卡住那一刻峰值或此刻内存已到上限的也照记 stalled、不重排（task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」（后两格推的，没复跑）
  TASK_SERVER_BREAK=booked-up-to-slice-total  上限与记账照改前收到那台的总上限、不减余量（task_server.py 的 host_capacity）：slice 里有一点回收不掉的用量时，记满一整台的永远起不来
      squeezed-at-total-is-cap-hit 那一格里“上限已到那台总上限”照 host_capacity 比：开关下比的是总上限本身，48G 的上限比不到、照挤掉原样重排
      「✗ task_server_lib.py 自证：backfill-when-head-blocked：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：aged-head-at-slice-total-starts：」、
      「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、「✗ task_server_lib.py 自证：unstarted-slice-uses-configured-total：」、「✗ task_server_lib.py 自证：margin-measured-per-host：」、
      「✗ task_server_lib.py 自证：explicit-wait-not-starved：」
  TASK_SERVER_BREAK=gpu-feature-any-subcommand  带 gpu 特性的 cargo 不看子命令一律要卡（改前的样子；task_server.py 的 cargo_uses_gpu）
      「✗ task_server_lib.py 自证：gpu-runs-only-when-code-runs：」
  TASK_SERVER_BREAK=no-run-needs-card  带 --no-run 的 cargo test / bench 照要卡（task_server.py 的 cargo_uses_gpu）
      「✗ task_server_lib.py 自证：gpu-runs-only-when-code-runs：」、「✗ task_server_lib.py 自证：build-family-declared：」（后一格推的，没复跑：同族那一条要卡而那一格没点名卡）
  TASK_SERVER_BREAK=stall-clock-reset-on-restart  服务起来对账在跑的尝试时「日志上一次涨」取此刻，不取日志的修改时刻（改前的样子；task_server.py 的 log_changed_at）
      「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」
  TASK_SERVER_BREAK=memory-cap-not-passed  不经环境 TASK_SERVER_MEMORY_CAP_BYTES 把这一次的上限交给命令（task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」
  TASK_SERVER_BREAK=source-repository-not-passed  不经环境 TASK_SERVER_SOURCE_REPOSITORY 把提交方那一份仓的根交给命令、派第二台时也不带（task_server.py 的 start_attempt、attempt_command）
      「✗ task_server_lib.py 自证：source-repository-passed-to-command：」
  TASK_SERVER_BREAK=suspect-unreported  CPU 满窗口不涨也不报疑似卡住（task_server.py 的 check_suspect）
      「✗ task_server_lib.py 自证：suspected-stall-notified：」、「✗ task_server_lib.py 自证：suspected-stall-card-busy-not-reported：」
  TASK_SERVER_BREAK=suspect-card-load-ignored  要卡的判疑似卡住不看它那张卡上有没有它的计算负载（task_server.py 的 check_suspect）
      「✗ task_server_lib.py 自证：suspected-stall-card-busy-not-reported：」
      「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」（推的，没复跑：卡上一直有负载的那一条 CPU 不涨先报疑似卡住，wait 退 231 不是 230）
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
      「✗ task_server_lib.py 自证：shard-keys-kept-apart：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、
      「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」、「✗ task_server_lib.py 自证：shard-pieces-estimated-by-own-history：」
  TASK_SERVER_BREAK=booked-at-peak  准入照改前按峰值记账、在跑的待涨量按记的峰值算（task_server.py 的 booked_at_cap 与 server_pending）
      「✗ task_server_lib.py 自证：cap-hit-rerun-from-record：」、「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、
      「✗ task_server_lib.py 自证：pinned-cap-by-host：」、「✗ task_server_lib.py 自证：admission-booked-at-cap：」、「✗ task_server_lib.py 自证：whole-and-fragment-alternate：」、「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」、
      「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、「✗ task_server_lib.py 自证：aged-tasks-do-not-hold-fitting-tasks：」、「✗ task_server_lib.py 自证：environment-prefix-stripped-from-key：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」、
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」、「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、
      「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、
      「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」
  TASK_SERVER_BREAK=health-judges-head  /health 照改前现判排头（现读库与余量、另判一套），不读排队线程算好的那一句（task_server.py 的 health 与 head_judged_apart）
      「✗ task_server_lib.py 自证：health-answers-while-resources-slow：」、「✗ task_server_lib.py 自证：health-head-cpu-gate-not-starved：」、「✗ task_server_lib.py 自证：health-head-cards-short-not-starved：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、
      「✗ task_server_lib.py 自证：progress-stall-reported：」
  TASK_SERVER_BREAK=python-entry-gpu-unrecognized  不认 python 入口 device-code-key.py run 的 --feature 带 gpu（task_server.py 的 python_entry_uses_gpu）
      「✗ task_server_lib.py 自证：python-entry-gpu-recognized：」
      「✗ task_server_lib.py 自证：temporary-service-heavy-refused：」

  TASK_SERVER_BREAK=alternation-disabled  整片与碎片不按台交替，照队列与回填排（task_server.py 的 Alternation）
      「✗ task_server_lib.py 自证：whole-and-fragment-alternate：」、「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、
      「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」
      「✗ task_server_lib.py 自证：aged-whole-keeps-group-sealed：」（量的：不封组，碎片照内存起）、「✗ task_server_lib.py 自证：card-whole-seals-while-card-busy：」（推的，没复跑：同样不封组）
      「✗ task_server_lib.py 自证：both-hosts-sealed-small-unestimated-starts：」、「✗ task_server_lib.py 自证：fragment-group-drops-finished：」（推的，没复跑：不封组，排队原因里没有封组那一句）
  TASK_SERVER_BREAK=whole-waits-for-empty-host  整片任务等那台服务起的尝试一条都没有才起（改前的样子；task_server.py 的 Alternation.blocked）
      「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」
  TASK_SERVER_BREAK=sealed-group-held-when-drained  封了组的那台腾空了、整片仍放不下时照改前一直封着（碎片永远等那条整片；task_server.py 的 Alternation.blocked）
      「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」
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
  TASK_SERVER_BREAK=registry-service-column-ignored  登记表 verdict-library-scripts.tsv 第三列写 prune-server 的族也不认、提交时照拒（task_server.py 的 verdict_library_refusal）
      「✗ task_server_lib.py 自证：verdict-library-family-refused：」
  TASK_SERVER_BREAK=cpu-gate-ignored  不判 CPU 闸：负载超过核数 × CPU_OVERSUBSCRIPTION 的台照样派（task_server.py 的 cpu_problem）
      「✗ task_server_lib.py 自证：cpu-gate-holds-busy-host：」
  TASK_SERVER_BREAK=unstarted-slice-estimated  slice 还没起的台照报告里估的总上限判，不换成多机配置里那台那一项（task_server.py 的 unstarted_slices_filled）
      「✗ task_server_lib.py 自证：unstarted-slice-uses-configured-total：」
  TASK_SERVER_BREAK=down-host-judged-never  多机配置里有一台此刻连不上时，只有它放得下的任务照改前判永远放不下（task_server.py 的 place）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
  TASK_SERVER_BREAK=card-fault-not-replaced  清场判出坏卡照改前整件判 card-fault 收尾，不停用那张卡、不换卡重排（task_server.py 的 settle_attempt 与 retry_decision）
      「✗ task_server_lib.py 自证：card-fault-replaced-on-another-card：」
      「✗ task_server_lib.py 自证：attention-card-replaced：」、「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」
  TASK_SERVER_BREAK=card-fault-assigned-card-quarantined  清场判出坏卡时停用的、告警与结局句点名的是这一次分到的那张，不读清场那一行的坏卡清单（改前的样子；task_server.py 的 finish_attempt 与 settle_attempt）
      「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」、「✗ task_server_lib.py 自证：card-fault-unlisted-quarantines-none：」
  TASK_SERVER_BREAK=failed-peak-ignored  只有判红尝试的命令照没记录取 4G，不以判红那一次的峰值 × 1.5 当下界（task_server.py 的 failed_peak_floor）
      「✗ task_server_lib.py 自证：failed-peak-floor：」
  TASK_SERVER_BREAK=environment-prefix-kept  命令键不剥 env 与不改跑法的环境变量前缀（key_words）
      「✗ task_server_lib.py 自证：environment-prefix-stripped-from-key：」
  TASK_SERVER_BREAK=workspace-writers-retried  改主工作区的命令撞了上限、被挤掉这类本会重排的结局照原样重排（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：workspace-writer-not-retried：」
  TASK_SERVER_BREAK=recorded-cap-floored  有记录的也抬到 4G（改前的样子）：钉值之外低于 4G 的上限与记账都兜到 4G，有记录的头一次、重排、撞过上限再交、改工作区的首趟都算
      （task_server.py 的 memory_floor_applied、workspace_writer_headroom）
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」（有记录的 200M 那一条开关下是 4096M）、
      「✗ task_server_lib.py 自证：retry-cap-not-floored：」（那一格头一次照记录取 371M、重排 557M，开关下都是 4096M）、
      「✗ task_server_lib.py 自证：cap-hit-rerun-from-record：」（推的，没复跑：小的一条头一遍 371M、再交 557M，开关下兜到 4096M、6144M；记录里只撞过一次的那条 371M 兜到 4096M）
  TASK_SERVER_BREAK=sealed-gap-not-backfilled  封组的那台照改前一条碎片都不放，预计用时不超过挡路时长的短任务也封着（task_server.py 的 Alternation.gap_fill）
      「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、
      「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：sealed-gap-room-takes-unestimated：」（后一格推的，没复跑）
      「✗ task_server_lib.py 自证：card-whole-seals-while-card-busy：」（推的，没复跑：封着组时有预计用时的短碎片也不插空）
      「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」
      「✗ task_server_lib.py 自证：both-hosts-sealed-small-unestimated-starts：」、「✗ task_server_lib.py 自证：fragment-group-drops-finished：」（推的，没复跑：小任务插不了空）
  TASK_SERVER_BREAK=sealed-gap-length-ignored  封组的那台有预计用时的就放，不比挡路时长（task_server.py 的 Alternation.gap_fill）
      「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、
      「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」（整片之外照样空着的那一份放不下的那几条，照时长也放进去了）
      （gap-fill-by-room-skips-length 推的不再红：整片之外照样空着的那一份不减封组之前起的碎片，五条在装组的还在跑时就照内存那一支起、记「不看预计用时」，轮不到时长那一支）
  TASK_SERVER_BREAK=blocker-remaining-unbounded  挡路的尝试没有预计用时或已跑过预计用时的，还要跑多久当成没有尽头，不按已跑算（task_server.py 的 Alternation.blocking_seconds）
      「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」
  TASK_SERVER_BREAK=gap-fill-room-ignored  封组的那台照改前一律比预计用时与挡路时长，排头整片起来之后照公式还放得下的碎片也看时长（task_server.py 的 Alternation.gap_fill）
      「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：sealed-gap-room-takes-unestimated：」
      「✗ task_server_lib.py 自证：both-hosts-sealed-small-unestimated-starts：」、「✗ task_server_lib.py 自证：fragment-group-drops-finished：」（推的，没复跑：没有预计用时的小任务封着）
  TASK_SERVER_BREAK=item-estimate-skipped  三条按项入口点了 --item 的照改前不起估算子任务，提交了就进队（task_server.py 的 submit）
      「✗ task_server_lib.py 自证：item-estimate-before-queue：」、「✗ task_server_lib.py 自证：item-estimate-summed-from-rates：」、
      「✗ task_server_lib.py 自证：item-estimate-failure-queues-parent：」、「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」
      「✗ task_server_lib.py 自证：item-estimate-cancelled：」
  TASK_SERVER_BREAK=item-done-unrecorded  尝试收尾时不读 ITEM_DONE 行、不记每项的两个速率（task_server.py 的 record_item_rates）
      「✗ task_server_lib.py 自证：item-done-rates-recorded：」
  TASK_SERVER_BREAK=item-estimate-by-command  点了 --item 的预计用时照改前按命令那一种取（同一命令键最近一次跑成的用时），不照 ITEM_PLAN 与两个速率相加（task_server.py 的 estimate_of）
      「✗ task_server_lib.py 自证：item-estimate-summed-from-rates：」、「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」
  TASK_SERVER_BREAK=item-rate-missing-skipped  点到的项缺速率（从没跑过、要现判而没有现判速率、要复用而没有复用速率）的当成 0 秒照加，不交 None（task_server.py 的 item_estimate）
      「✗ task_server_lib.py 自证：item-estimate-summed-from-rates：」
  TASK_SERVER_BREAK=estimate-failure-holds-parent  估算子任务没成（退非 0）或没打 ITEM_PLAN 行时父任务一直在估算、不进队（task_server.py 的 drive_item_estimate）
      「✗ task_server_lib.py 自证：gpu-entry-scripts-recognized：」、「✗ task_server_lib.py 自证：item-estimate-failure-queues-parent：」
  TASK_SERVER_BREAK=estimate-basis-unreported  服务的回复里不交 estimate_basis（这次预计用时是按命令、按项还是没有；task_server.py 的 summary），客户端没有依据可打说明
      「✗ task_server_lib.py 自证：estimate-basis-command-sentence：」、「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」、
      「✗ task_server_lib.py 自证：estimate-basis-none-sentence：」
  TASK_SERVER_BREAK=estimate-basis-sentence-fixed  客户端 task status <号> 不看 estimate_basis，预计用时后面一律打按命令那一句（改前的样子；task 的 status）
      「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」、「✗ task_server_lib.py 自证：estimate-basis-none-sentence：」
  TASK_SERVER_BREAK=shard-plan-merge-unestimated  分片的计划与合并两步照改前没有预计用时（task_server.py 的 estimate_of）
      「✗ task_server_lib.py 自证：shard-plan-and-merge-estimated-by-command：」
  TASK_SERVER_BREAK=shard-pieces-unestimated  分片的各片照改前没有预计用时（task_server.py 的 estimate_of）
      「✗ task_server_lib.py 自证：shard-pieces-estimated-by-own-history：」
  TASK_SERVER_BREAK=attention-queue-unreported  排队过久不记成要处置的提醒，wait 一直排下去不叫醒（task_server.py 的 attention_items）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」
  TASK_SERVER_BREAK=attention-overrun-unreported  跑到记录用时很多倍不记成要处置的提醒（task_server.py 的 attention_items）
      「✗ task_server_lib.py 自证：attention-overrun：」
  TASK_SERVER_BREAK=attention-card-replaced-unreported  换卡重排不记成要处置的提醒（task_server.py 的 attention_items）
      「✗ task_server_lib.py 自证：attention-card-replaced：」、「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」
  TASK_SERVER_BREAK=attention-finished-unreported  任务已有结局时 wait 不带提醒（换卡重排跑成的那一次没人叫；task_server.py 的 wait 接口）
      「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」（后一格推的，没复跑）
  TASK_SERVER_BREAK=attention-cleanup-unreported  清场没收尾（cleanup_pending）不记成要处置的提醒（task_server.py 的 attention_items）
      「✗ task_server_lib.py 自证：attention-cleanup-pending：」
  TASK_SERVER_BREAK=attention-shard-advance-unreported  分片父任务推不动不记成要处置的提醒（task_server.py 的 drive_sharded_parents 与 attention_items）
      「✗ task_server_lib.py 自证：attention-shard-advance-failed：」
  TASK_SERVER_BREAK=attention-reported-every-wait  同一条任务同一种、同一档的提醒每起一次 wait 都叫（task_server.py 的 unreported_attention）
      「✗ task_server_lib.py 自证：attention-reported-once：」
  TASK_SERVER_BREAK=temporary-service-build-unrecognized  不带 --server-binary 的临时 prune-server 包装不当会编的算，本机不给编译目录（改前的样子；temporary_service_builds_server）
      「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」
  TASK_SERVER_BREAK=judge-error-blocks-queue  一条这一轮判不了放不放得下时照改前挡住它后面的整队（task_server.py 的 place_queued_into）
      「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」
  TASK_SERVER_BREAK=nfs-unchecked  不判 NFS 共享，没写 NFS_SHARED_ROOTS、导出或挂载不过也照起（nfs_share_report）
      「✗ task_server_lib.py 自证：build-family-cap-hit-raises：」、「✗ task_server_lib.py 自证：build-family-peak-windowed：」、「✗ task_server_lib.py 自证：pause-allows-one-session：」、「✗ task_server_lib.py 自证：pause-allow-waiting-text：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」、「✗ task_server_lib.py 自证：resume-clears-allowed-session：」、
      「✗ task_server_lib.py 自证：pause-allow-survives-restart：」、「✗ task_server_lib.py 自证：pause-allow-reported：」、「✗ task_server_lib.py 自证：pause-holds-new-attempts：」、「✗ task_server_lib.py 自证：pause-lets-running-finish：」、
      「✗ task_server_lib.py 自证：resume-releases-queue：」、「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：pause-not-starved：」、「✗ task_server_lib.py 自证：pause-waits-for-starting-attempt：」、
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-overrun：」、「✗ task_server_lib.py 自证：attention-card-replaced：」、
      「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：attention-shard-advance-failed：」、「✗ task_server_lib.py 自证：attention-reported-once：」、
      「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」、「✗ task_server_lib.py 自证：cap-hit-rerun-from-record：」、「✗ task_server_lib.py 自证：exit-code-passthrough：」、
      「✗ task_server_lib.py 自证：restart-keeps-running：」、「✗ task_server_lib.py 自证：start-once：」、「✗ task_server_lib.py 自证：queue-holds-unplaceable：」、「✗ task_server_lib.py 自证：arrival-order：」、
      「✗ task_server_lib.py 自证：prediction-computed-once：」、「✗ task_server_lib.py 自证：pass-reads-resources-once：」、「✗ task_server_lib.py 自证：pass-booking-carried：」、「✗ task_server_lib.py 自证：launch-rechecked：」、
      「✗ task_server_lib.py 自证：every-command-reaches-peer：」、「✗ task_server_lib.py 自证：backfill-when-head-blocked：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：unit-start-failed：」、
      「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：unrecorded-default-small：」、「✗ task_server_lib.py 自证：unrecorded-cap-growth：」、
      「✗ task_server_lib.py 自证：unrecorded-burst-balanced：」、「✗ task_server_lib.py 自证：retry-kinds：」、「✗ task_server_lib.py 自证：gpu-highest-free-card：」、「✗ task_server_lib.py 自证：gpu-entry-scripts-recognized：」、
      「✗ task_server_lib.py 自证：cancel-own-session-only：」、「✗ task_server_lib.py 自证：queued-revalidated-on-restart：」、「✗ task_server_lib.py 自证：forged-session：」、「✗ task_server_lib.py 自证：environment-whitelist：」、
      「✗ task_server_lib.py 自证：nested-dispatch-refused：」、「✗ task_server_lib.py 自证：resource-record：」、「✗ task_server_lib.py 自证：operation-log：」、「✗ task_server_lib.py 自证：unclaimed-finished：」、
      「✗ task_server_lib.py 自证：retry-keeps-earlier-attempts：」、「✗ task_server_lib.py 自证：build-directory-outside-repository：」、「✗ task_server_lib.py 自证：retry-build-directory-reused：」、「✗ task_server_lib.py 自证：thirty-second-tick-prompt：」、
      「✗ task_server_lib.py 自证：starved-head-reported：」、「✗ task_server_lib.py 自证：blocked-head-not-starved：」、「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」、
      「✗ task_server_lib.py 自证：unmeasured-not-zero：」、「✗ task_server_lib.py 自证：queued-estimate：」、「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」、「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」、
      「✗ task_server_lib.py 自证：clean-failure-explained：」、「✗ task_server_lib.py 自证：stalled-attempt-stopped：」、「✗ task_server_lib.py 自证：cpu-unread-not-stalled：」、「✗ task_server_lib.py 自证：running-status-shows-progress：」、
      「✗ task_server_lib.py 自证：stall-waits-for-grace：」、「✗ task_server_lib.py 自证：computing-quietly-not-stalled：」、「✗ task_server_lib.py 自证：overrun-quiet-log-stopped：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、
      「✗ task_server_lib.py 自证：simulated-two-host-workload：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、
      「✗ task_server_lib.py 自证：aged-head-at-slice-total-starts：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、「✗ task_server_lib.py 自证：peer-starts-throttled：」、
      「✗ task_server_lib.py 自证：attempt-unit-nofile-raised：」、「✗ task_server_lib.py 自证：queued-shows-waiting-reason：」、「✗ task_server_lib.py 自证：failed-run-estimated：」、「✗ task_server_lib.py 自证：unfinished-peak-ignored：」、
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：balanced-when-both-fit：」、「✗ task_server_lib.py 自证：peer-when-local-full：」、
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：threads-fixed：」、
      「✗ task_server_lib.py 自证：shard-retry-isolated：」、「✗ task_server_lib.py 自证：task-log：」、「✗ task_server_lib.py 自证：cancel-queued：」、「✗ task_server_lib.py 自证：cancel-finished：」、
      「✗ task_server_lib.py 自证：cancel-sharded-in-plan：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：shard-plan-failed：」、「✗ task_server_lib.py 自证：shard-merge-failed：」、
      「✗ task_server_lib.py 自证：unsharded-arguments：」、「✗ task_server_lib.py 自证：gpu-without-cards-unmet：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、「✗ task_server_lib.py 自证：gpu-runs-only-when-code-runs：」、
      「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」、「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」、「✗ task_server_lib.py 自证：source-repository-passed-to-command：」、「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」、
      「✗ task_server_lib.py 自证：card-labels-by-address：」、「✗ task_server_lib.py 自证：peer-admission-marked：」、「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：suspected-stall-notified：」、
      「✗ task_server_lib.py 自证：suspected-stall-card-busy-not-reported：」、「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：pinned-cap-unreadable-refused：」、「✗ task_server_lib.py 自证：pinned-cap-by-host：」、
      「✗ task_server_lib.py 自证：by-host-missing-host-refused：」、「✗ task_server_lib.py 自证：by-host-outside-address-refused：」、「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」、
      「✗ task_server_lib.py 自证：admission-booked-at-cap：」、「✗ task_server_lib.py 自证：health-answers-while-resources-slow：」、「✗ task_server_lib.py 自证：python-entry-gpu-recognized：」、「✗ task_server_lib.py 自证：gpu-argument-declaration-recognized：」、
      「✗ task_server_lib.py 自证：leftover-processes-stopped：」、「✗ task_server_lib.py 自证：long-run-failure-noted：」、「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」、「✗ task_server_lib.py 自证：whole-and-fragment-alternate：」、
      「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」、「✗ task_server_lib.py 自证：drained-sealed-group-reopens：」、「✗ task_server_lib.py 自证：ended-attempt-settled-after-restart：」、「✗ task_server_lib.py 自证：aged-tasks-do-not-hold-fitting-tasks：」、
      「✗ task_server_lib.py 自证：cpu-gate-holds-busy-host：」、「✗ task_server_lib.py 自证：unstarted-slice-uses-configured-total：」、「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：card-fault-replaced-on-another-card：」、
      「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」、「✗ task_server_lib.py 自证：card-fault-unlisted-quarantines-none：」、「✗ task_server_lib.py 自证：failed-peak-floor：」、「✗ task_server_lib.py 自证：environment-prefix-stripped-from-key：」、
      「✗ task_server_lib.py 自证：verdict-library-family-refused：」、「✗ task_server_lib.py 自证：workspace-writer-not-retried：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」、「✗ task_server_lib.py 自证：recorded-cap-not-floored：」、
      「✗ task_server_lib.py 自证：sealed-gap-takes-short-task：」、「✗ task_server_lib.py 自证：sealed-gap-holds-longer-task：」、「✗ task_server_lib.py 自证：unestimated-blocker-counted-by-elapsed：」、「✗ task_server_lib.py 自证：margin-measured-per-host：」、
      「✗ task_server_lib.py 自证：gap-fill-by-room-skips-length：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、「✗ task_server_lib.py 自证：item-estimate-before-queue：」、「✗ task_server_lib.py 自证：item-done-rates-recorded：」、
      「✗ task_server_lib.py 自证：item-estimate-summed-from-rates：」、「✗ task_server_lib.py 自证：item-estimate-failure-queues-parent：」、「✗ task_server_lib.py 自证：item-estimate-cancelled：」、「✗ task_server_lib.py 自证：estimate-by-folded-command-key：」、
      「✗ task_server_lib.py 自证：estimate-basis-command-sentence：」、「✗ task_server_lib.py 自证：estimate-basis-items-sentence：」、「✗ task_server_lib.py 自证：estimate-basis-none-sentence：」、「✗ task_server_lib.py 自证：shard-plan-and-merge-estimated-by-command：」、
      「✗ task_server_lib.py 自证：shard-pieces-estimated-by-own-history：」、「✗ task_server_lib.py 自证：margin-unmeasured-not-placed：」、「✗ task_server_lib.py 自证：pin-cap-refused-within-margin：」、「✗ task_server_lib.py 自证：peer-booked-like-local：」、
      「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」、「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」、「✗ task_server_lib.py 自证：explicit-wait-not-starved：」、
      「✗ task_server_lib.py 自证：nfs-share-required：」、「✗ task_server_lib.py 自证：nfs-share-verified：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、「✗ task_server_lib.py 自证：reason-line-per-type：」、
      「✗ task_server_lib.py 自证：cancel-start-race：」、「✗ task_server_lib.py 自证：sharded-progress-recovered：」、「✗ task_server_lib.py 自证：attempt-claim-atomic：」、「✗ task_server_lib.py 自证：sharded-cancel-retried：」、
      「✗ task_server_lib.py 自证：association-column-verified：」、「✗ task_server_lib.py 自证：start-timeout-read-back：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」、
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：prune-server-unavailable-outcome：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、「✗ task_server_lib.py 自证：temporary-service-heavy-refused：」、
      「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：finished-build-removed：」、「✗ task_server_lib.py 自证：nfs-probe-concurrent-runs：」、
      「✗ task_server_lib.py 自证：gpu-binary-declaration-recognized：」、「✗ task_server_lib.py 自证：short-command-peak-recorded：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、「✗ task_server_lib.py 自证：stop-ends-probe-children：」、
      「✗ task_server_lib.py 自证：failure-scene-kept-until-collected：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」、
      「✗ task_server_lib.py 自证：cancel-after-natural-end：」、「✗ task_server_lib.py 自证：failed-start-not-counted：」、「✗ task_server_lib.py 自证：child-rows-verified：」、「✗ task_server_lib.py 自证：shard-siblings-share-peaks：」、
      「✗ task_server_lib.py 自证：build-family-peak-shared：」、「✗ task_server_lib.py 自证：build-family-declared：」、「✗ task_server_lib.py 自证：build-family-from-arguments：」、「✗ task_server_lib.py 自证：shard-child-key-folded-by-words：」、「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」、
      「✗ task_server_lib.py 自证：workspace-write-complete-not-reported：」、「✗ task_server_lib.py 自证：gpu-two-cards-same-host：」、「✗ task_server_lib.py 自证：gpu-three-cards-only-where-enough：」、「✗ task_server_lib.py 自证：gpu-cards-short-never-fits：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」、
      「✗ task_server_lib.py 自证：aged-whole-keeps-group-sealed：」、「✗ task_server_lib.py 自证：attention-quiet-busy-log：」、「✗ task_server_lib.py 自证：card-whole-seals-while-card-busy：」（最后一格推的，没复跑：
      量的那一趟它因别的原因红着，看不出这条开关红不红它；起服务提交任务的格在这条开关下都红）
      「✗ task_server_lib.py 自证：whole-workspace-cargo-refused：」、「✗ task_server_lib.py 自证：workspace-writer-initial-cap-headroom：」、
      「✗ task_server_lib.py 自证：both-hosts-sealed-small-unestimated-starts：」、「✗ task_server_lib.py 自证：fragment-group-drops-finished：」（这四格推的，没复跑：同上，起服务提交任务）、
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcome-failed：」、「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：inbox-unreachable-task-runs：」、「✗ task_server_lib.py 自证：unclaimed-results-warned：」、
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」（推的，没复跑：同上，起服务提交任务）
      「✗ task_server_lib.py 自证：card-zero-utilization-stalled：」（推的，没复跑：同上，起服务提交任务）
      「✗ task_server_lib.py 自证：cancelled-before-peer-trip-built：」（推的，没复跑：同上，起服务提交任务）
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=nfs-probe-one-way  两向探针只做本机→第二台那一向，第二台写的本机读不回也照过（nfs_share_report）
      「✗ task_server_lib.py 自证：nfs-share-verified：」
  TASK_SERVER_BREAK=nfs-probe-at-root  根是 git 工作树也把探针写在根下（nfs_probe_directory），红在 nfs-share-verified 一格
      「✗ task_server_lib.py 自证：nfs-share-verified：」
  TASK_SERVER_BREAK=finished-build-kept  任务记下结局之后照改前留着各次尝试的编译目录、等提交方 task clean（服务起来时也不扫；task_server.py 的 remove_finished_build_directories）
      「✗ task_server_lib.py 自证：finished-build-removed：」
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
      「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：pinned-cap-unreadable-refused：」、「✗ task_server_lib.py 自证：card-fault-quarantines-named-card：」、「✗ task_server_lib.py 自证：card-fault-unlisted-quarantines-none：」、
      「✗ task_server_lib.py 自证：reason-line-per-type：」
  TASK_SERVER_BREAK=clean-failure-unexplained  task clean 没删成时只说没删干净，不写是哪一份、为什么，操作日志也不写原因（task 的 clean、task_server.py 的 clean）
      「✗ task_server_lib.py 自证：clean-failure-explained：」
  TASK_SERVER_BREAK=shared-roots-not-passed  不经环境 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT 把两个共享根交给命令，派第二台时也不带（改前的样子；task_server.py 的 start_attempt、attempt_command）
      「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」
  TASK_SERVER_BREAK=peer-admission-unmarked  派第二台的那一次不给 multi-host-run.sh --peer 设 PEER_ADMITTED_VARIABLE（改前的样子；task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：peer-admission-marked：」
  TASK_SERVER_BREAK=disk-floor-unchecked  不看各台盘余量，不到下限也照派（改前的样子；task_server.py 的 Resources.disk_problem）
      「✗ task_server_lib.py 自证：peer-disk-floor：」
  TASK_SERVER_BREAK=peer-tree-copied  派第二台照改前拷树：加 --send <工作目录> <树里的落点>、命令换到落点里起（task_server.py 的 attempt_command；假第二台照真的那样认不出 --send、退 2）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」、
      「✗ task_server_lib.py 自证：every-command-reaches-peer：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」、
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：simulated-two-host-workload：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」、
      「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、「✗ task_server_lib.py 自证：peer-starts-throttled：」、「✗ task_server_lib.py 自证：peer-when-local-full：」、
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」、
      「✗ task_server_lib.py 自证：source-repository-passed-to-command：」、「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」、「✗ task_server_lib.py 自证：card-labels-by-address：」、「✗ task_server_lib.py 自证：peer-admission-marked：」、
      「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」、「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」、
      「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：verdict-library-family-refused：」、「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：margin-unmeasured-not-placed：」、
      「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」、「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、
      「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、
      「✗ task_server_lib.py 自证：failed-start-not-counted：」、「✗ task_server_lib.py 自证：retry-build-directory-reused：」、「✗ task_server_lib.py 自证：gpu-three-cards-only-where-enough：」
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
  TASK_SERVER_BREAK=association-column-unchecked  读回不比关联列 task_id 与正文（改前的样子，TS-05；task_server.py 的 Store.verified）
      「✗ task_server_lib.py 自证：association-column-verified：」
  TASK_SERVER_BREAK=child-rows-unverified  children_of 丢掉校验位、名下校验不过的子任务行照收（task_server.py 的 children_of：status、clean、取消分片的整件、推父任务都经它读）
      「✗ task_server_lib.py 自证：child-rows-verified：」、「✗ task_server_lib.py 自证：association-column-verified：」
  TASK_SERVER_BREAK=start-timeout-as-failure  systemd-run 超时当确定没起、记 unit-start-failed（改前的样子，TS-06；task_server.py 的 start_unit）
      「✗ task_server_lib.py 自证：start-timeout-read-back：」
  TASK_SERVER_BREAK=build-disk-unpredicted  派活不看编译目录占盘的预测（改前的样子；task_server.py 的 build_disk_problem）
      「✗ task_server_lib.py 自证：build-disk-predicted：」
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
  TASK_SERVER_BREAK=temporary-service-heavy-accepted  提交时不判临时 prune-server 包装里的原命令，包着会编的 cargo、要卡的照收（改前的样子；task_server.py 的 placement_refusal 与 temporary_service_heavy_refusal）
      「✗ task_server_lib.py 自证：temporary-service-heavy-refused：」
  TASK_SERVER_BREAK=gpu-feature-names-unread  带特性的 cargo 照改前只认 gpu 与 <包>/gpu，tests-on-a-real-gpu 认成不要卡（task_server.py 的 cargo_feature_gpu）
      「✗ task_server_lib.py 自证：gpu-runs-only-when-code-runs：」
  TASK_SERVER_BREAK=gpu-argument-declaration-ignored  不读脚本文件头的 # task-server-gpu-arguments:，按参数才要卡的脚本认成不要卡（declared_gpu_arguments）
      「✗ task_server_lib.py 自证：gpu-argument-declaration-recognized：」
      「✗ task_server_lib.py 自证：gpu-binary-declaration-recognized：」
      「✗ task_server_lib.py 自证：gpu-two-cards-same-host：」、「✗ task_server_lib.py 自证：gpu-three-cards-only-where-enough：」、「✗ task_server_lib.py 自证：gpu-cards-short-never-fits：」（推的，没复跑）
  TASK_SERVER_BREAK=estimate-by-exact-command  预计用时照改前只认命令逐词相同的尝试，路径不同而命令键相同的取不到（任务 7585；task_server.py 的 command_estimate）
      「✗ task_server_lib.py 自证：estimate-by-folded-command-key：」
  TASK_SERVER_BREAK=binary-gpu-declaration-ignored  编出来的二进制不读同名 src/bin 源文件头的 // task-server-gpu-arguments:，要卡的裸二进制认成不要卡（declared_gpu_binary）
      「✗ task_server_lib.py 自证：gpu-binary-declaration-recognized：」
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
      「✗ task_server_lib.py 自证：build-family-peak-shared：」、「✗ task_server_lib.py 自证：build-family-declared：」（推的，没复跑：同族那一条的峰值记不下）、
      「✗ task_server_lib.py 自证：shard-siblings-peak-windowed：」（推的：各片跑成的峰值记不下）、「✗ task_server_lib.py 自证：build-family-from-arguments：」（推的，没复跑：同族那一条的峰值记不下）
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
  TASK_SERVER_BREAK=shard-siblings-abandoned  一片没成时照改前当场记整件的结局，在跑的兄弟片没人收（任务 5850 的样子；task_server.py 的 drive_parent_phase）
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」
  TASK_SERVER_BREAK=shard-cancel-ends-whole  人手只取消一片时照改前当成一片没成，整件收尾、不合并（task_server.py 的 cancel 不记 cancel_origin）
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=shard-missing-unnamed  合并那一步不设 TASK_SERVER_SHARD_MISSING，缺了哪几片不交给合并（task_server.py 的 shard_environment）
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=parent-cancel-spares-running  取消分片的整件照兄弟片没成那一路只撤在排的子任务，在跑的不停（task_server.py 的 cancel_children）
      「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
      「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：item-estimate-cancelled：」、「✗ task_server_lib.py 自证：sharded-cancel-retried：」
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
  TASK_SERVER_BREAK=shard-siblings-unshared  分片的一片不看同一计划里计划那一步与兄弟片的峰值，各片照自己的键从 4G 爬（改前的样子；task_server.py 的 shard_family_peak）
      「✗ task_server_lib.py 自证：shard-siblings-share-peaks：」、「✗ task_server_lib.py 自证：shard-siblings-peak-windowed：」
  TASK_SERVER_BREAK=build-family-ignored  提交时不记编译族、定额度不看族，会编的 cargo 与声明了族的入口同键没记录只从 4G 起（改前的样子；task_server.py 的 build_family_of、build_family_peak）
      「✗ task_server_lib.py 自证：build-family-peak-shared：」、「✗ task_server_lib.py 自证：build-family-declared：」、「✗ task_server_lib.py 自证：build-family-from-arguments：」、「✗ task_server_lib.py 自证：build-family-cap-hit-raises：」、「✗ task_server_lib.py 自证：build-family-peak-windowed：」
  TASK_SERVER_BREAK=family-arguments-unfilled  编译族声明里的 $<n>、$<选项名> 不照命令参数填，照字面算族（改前的样子；filled_family_words）：证红按 <crate> 归不进同族、照 4G 起
      「✗ task_server_lib.py 自证：build-family-from-arguments：」
  TASK_SERVER_BREAK=family-cap-hit-not-counted  编译族的窗口只收跑成的那几次的编译段峰值，撞了上限的那几次（编完了才撞的）的编译段峰值不收（task_server.py 的 build_family_peak、attempt_measurement）
      「✗ task_server_lib.py 自证：build-family-cap-hit-raises：」
  TASK_SERVER_BREAK=retry-build-directory-fresh  撞上限重排时当场删这一次的编译目录，下一次新建、从零编（改前的样子；task_server.py 的 settle_attempt、start_attempt）
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」、「✗ task_server_lib.py 自证：peer-retry-build-directory-reused：」（后一格推的，没复跑）
  TASK_SERVER_BREAK=shard-key-from-joined-command  分片子任务的键照改前用空格接父命令的词再按空格切，条目名里带空格的后半截漏进键里（task_server.py 的 create_child）
      「✗ task_server_lib.py 自证：shard-child-key-folded-by-words：」
  TASK_SERVER_BREAK=workspace-write-interrupted-unreported  改主工作区的命令没正常结束也不报（改前的样子；task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」
  TASK_SERVER_BREAK=apply-outcome-ignored  不读 apply-writer-patch.py 的结局文件，打全了的也照可能写了一半报（task_server.py 的 workspace_write_judged）
      「✗ task_server_lib.py 自证：workspace-write-complete-not-reported：」
  TASK_SERVER_BREAK=gpu-card-count-ignored  不读源文件头的张数声明，要卡的一律一张（改前的样子；task_server.py 的 gpu_cards_wanted）
      「✗ task_server_lib.py 自证：gpu-two-cards-same-host：」、「✗ task_server_lib.py 自证：gpu-three-cards-only-where-enough：」、「✗ task_server_lib.py 自证：gpu-cards-short-never-fits：」
  TASK_SERVER_BREAK=history-peak-ignored  头一次与重排都不看这把键近几次里最高的一次（头一次只照定额度那一段、重排只照自己那一次 × 1.5 一级级乘；task_server.py 的 history_raised_prediction）
      「✗ task_server_lib.py 自证：history-peak-retry-jumps：」、「✗ task_server_lib.py 自证：build-family-cap-hit-raises：」（后一格推的，没复跑：重排那一次另看编译族的那一段在 history_raised_prediction 里）
  TASK_SERVER_BREAK=history-peak-unwindowed  起点照改前不截近 RECORD_WINDOW 次、照这把键全部记录里最高的一次（只升不降；兄弟片与编译族那两段同样不截；task_server.py 的 window_maximum，key_recent_peak、shard_family_peak、build_family_peak 都经它）
      「✗ task_server_lib.py 自证：history-peak-first-attempt：」、「✗ task_server_lib.py 自证：shard-siblings-peak-windowed：」、「✗ task_server_lib.py 自证：build-family-peak-windowed：」
  TASK_SERVER_BREAK=whole-seals-every-host  一条整片照改前在它算整片的每一台上都封组等腾空，只起在一台上却封住两台（任务 7585；task_server.py 的 Alternation.waiting_hosts）
      「✗ task_server_lib.py 自证：whole-seals-one-host：」
  TASK_SERVER_BREAK=card-whole-counts-free-cards-only  要卡的整片照改前只在空卡凑得够它张数的台上算整片：卡一直有人占的台从不为它封组，碎片一直占着内存（任务 7934；task_server.py 的 Alternation.waiting_hosts）
      「✗ task_server_lib.py 自证：card-whole-seals-while-card-busy：」
  TASK_SERVER_BREAK=at-cap-busy-cpu-kept-running  日志满窗口不涨、此刻内存贴着上限而 CPU 在涨的照改前当在算、照跑（任务 7723；task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」
  TASK_SERVER_BREAK=at-cap-fraction-ignored  判贴着上限不比上限：读得到此刻内存就当贴着上限（task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」
  TASK_SERVER_BREAK=peer-memory-line-unread  第二台不读日志里的 MULTI_HOST_RUN_PEER_MEMORY 那一行，此刻内存与峰值都交 None（那一行照旧不算日志在涨；task_server.py 的 PeerMachine.memory_now）
      「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」
  TASK_SERVER_BREAK=aged-whole-not-counted  满时限的整片照改前不算整片、不为它封组，封着的组一满时限就撤掉（任务 7934；task_server.py 的 Alternation）
      「✗ task_server_lib.py 自证：aged-whole-keeps-group-sealed：」
  TASK_SERVER_BREAK=aged-fragment-bypasses-seal  满时限的碎片照改前不过交替，封了的组不挡它（task_server.py 的 place_queued_into）
      「✗ task_server_lib.py 自证：aged-whole-keeps-group-sealed：」
  TASK_SERVER_BREAK=attention-quiet-busy-unreported  日志长期不涨而服务没停它不记成要处置的提醒（task_server.py 的 quiet_log_items）
      「✗ task_server_lib.py 自证：attention-quiet-busy-log：」
  TASK_SERVER_BREAK=io-wait-ignored  判卡住不看 D 状态：日志与 CPU 都满窗口不涨、每一轮都有进程在等 IO 的也停掉、记 stalled（改前的样子；任务 8608、8609、8611、8619；
      task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」、「✗ task_server_lib.py 自证：io-wait-peer-not-stalled：」
  TASK_SERVER_BREAK=attention-io-wait-unreported  卡在 IO 上而服务没停它不记成要处置的提醒（task_server.py 的 io_wait_items）
      「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」
  TASK_SERVER_BREAK=stall-card-load-ignored  判卡住不看卡：分到了卡、日志与 CPU 都满窗口不涨而每一轮卡上都有它的计算负载的也停掉、记 stalled（改前的样子；任务 8519、8520；
      task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」、「✗ task_server_lib.py 自证：card-zero-utilization-stalled：」
  TASK_SERVER_BREAK=card-presence-counts-as-busy  判卡住不看卡的利用率：分到了卡、日志与 CPU 都满窗口不涨、每一轮卡上都有它的计算进程而利用率每一轮都读到 0 的也免停
      （task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：card-zero-utilization-stalled：」
      「✗ task_server_lib.py 自证：card-idle-polling-stalled：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=attention-gpu-busy-quiet-unreported  卡上有它的计算进程、日志与 CPU 都不涨而服务没停它不记成要处置的提醒（task_server.py 的 gpu_busy_quiet_items）
      「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」
  TASK_SERVER_BREAK=quiet-log-busy-any-exemption  日志长期不涨的提醒不看服务为什么没停它：按 D 状态、卡上负载免停的也报 quiet-log-busy（改前的样子；task_server.py 的 quiet_log_items）
      「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」、「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」
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
  TASK_SERVER_BREAK=workspace-writer-headroom-skipped  改主工作区的命令首趟照改前只看同键窗口 × 1.5，不加它要起的验证命令那把键、不兜到 4G（任务 8367；
      task_server.py 的 workspace_writer_headroom）
      「✗ task_server_lib.py 自证：workspace-writer-initial-cap-headroom：」、「✗ task_server_lib.py 自证：cap-hit-rerun-from-record：」
  TASK_SERVER_BREAK=gap-fill-room-counts-all-running  插空的内存那一支照改前连封组之前起的碎片一起减（room_by_the_formula 减排头的记账），碎片跑完之前没有预计用时的一条都放不进
      （任务 8370；task_server.py 的 Alternation.gap_fill）
      「✗ task_server_lib.py 自证：both-hosts-sealed-small-unestimated-starts：」、「✗ task_server_lib.py 自证：gap-fill-room-exhausted-checks-length：」、
      「✗ task_server_lib.py 自证：fragment-group-drops-finished：」（gap-fill-room-exhausted-checks-length 照改前只放得下一条、fragment-group-drops-finished 的小任务插不了空，
      这两格推的，没复跑）
  TASK_SERVER_BREAK=fragment-group-never-drops  组的账照改前只加不减：封着组时插空起的也记进账、这一组还在跑的那几条跑完也不剪（任务 8370 那一回「已装到 95.1 GiB（界 41.6 GiB）」；
      task_server.py 的 Alternation.__init__、started）
      「✗ task_server_lib.py 自证：fragment-group-drops-finished：」
  TASK_SERVER_BREAK=repository-target-bypass-unwarned  提交时不判命令与入口脚本取仓根 target/ 下的二进制，提交回复、日志头与 status 都不打严厉警告
      （改前的样子：任务 7937–7944 取了仓根 target/ 下合并前的旧二进制；task_server.py 的 repository_target_bypass）
      「✗ task_server_lib.py 自证：repository-target-bypass-warned：」
  TASK_SERVER_BREAK=cancelled-artifacts-kept  取消之后不删各次尝试的编译目录与现场目录、不核第二台那一趟的本地目录，取消的回复与日志里不报删前删后（改前的样子：
      取消之后编译目录与现场等提交它的会话另下删除指令；task_server.py 的 remove_cancelled_artifacts）
      「✗ task_server_lib.py 自证：cancelled-artifacts-removed：」
  TASK_SERVER_BREAK=cancel-not-started-receipt-unread  取消之后核第二台那一趟的本地目录不读清理回执：回执还是 not-started、目录还没建的，日志里没有 ⑥ 那一行也算没删成、
      cancel 退 5（改前的样子；task_server.py 的 cancelled_attempt_artifacts）
      「✗ task_server_lib.py 自证：cancelled-before-peer-trip-built：」
  TASK_SERVER_BREAK=inbox-not-sent  服务一条消息都不往会话的收件 socket 发（task_server.py 的 send_to_session），发不出去的告警也不写
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcome-failed：」、「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：inbox-unreachable-task-runs：」、
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」、「✗ task_server_lib.py 自证：over-two-hours-bounced：」、「✗ task_server_lib.py 自证：long-run-confirmed-not-bounced：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=inbox-auth-omitted  发消息不发认证行、只发 user 那一行（task_server.py 的 post_to_inbox）
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcome-failed：」、「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」、「✗ task_server_lib.py 自证：over-two-hours-bounced：」、「✗ task_server_lib.py 自证：long-run-confirmed-not-bounced：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=inbox-reported-every-round  发消息那一路不看报过的档与段，每一轮照发（task_server.py 的 push_attention、push_suspects）
      「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」
  TASK_SERVER_BREAK=pushed-outcome-not-collected  结局发出去了也不记已取（task_server.py 的 push_outcomes）
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcomes-resent-after-error：」
  TASK_SERVER_BREAK=inbox-pending-dropped-on-error  发结局那一轮处理到某一条抛错时不把它与后面还没处理的号放回待发，下一轮不补发（改前的样子；task_server.py 的 push_outcomes）
      「✗ task_server_lib.py 自证：inbox-outcomes-resent-after-error：」
  TASK_SERVER_BREAK=health-unwatched  服务不盯自己的健康、不发「服务健康」（task_server.py 的 watch_health）
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=health-alerted-every-round  同一样服务健康对同一个会话每一轮照发，不记发过的（task_server.py 的 watch_health）
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=unclaimed-unwarned  交任务时不判会话名下有没有结局满时限没人取的，不打严厉警告（task_server.py 的 unclaimed_before）
      「✗ task_server_lib.py 自证：unclaimed-results-warned：」
  TASK_SERVER_BREAK=leftovers-error-uncaught  attempt_runner 停单元里留下的进程时抛的异常照改前不隔离，result.json 写不出、成功的任务被判 launcher-lost
      （records/2026-11-03-task-server与prune-server深入对攻审核.md「TS-02」；说明里的 leftovers_error_uncaught，attempt_runner.py 的 main）
      「✗ task_server_lib.py 自证：leftovers-error-isolated：」
  TASK_SERVER_BREAK=inbox-attention-per-shard  发给会话的要处置的提醒照改前逐片各发一条、不合、不压一轮（task_server.py 的 attention_groups、push_attention）
      「✗ task_server_lib.py 自证：inbox-shard-attention-merged：」
  TASK_SERVER_BREAK=memory-peak-with-page-cache  本机记的峰值与判贴着上限照改前读 scope 的 memory.peak、memory.current（含页缓存；任务 8853；
      attempt_runner.py 的 Sampler 照说明里的 memory_peak_with_page_cache，task_server.py 的 scope_memory_peak、scope_memory_current）
      「✗ task_server_lib.py 自证：page-cache-not-counted-as-process-memory：」、「✗ task_server_lib.py 自证：short-command-peak-recorded：」
  TASK_SERVER_BREAK=attention-reported-memory-only  报过的提醒档照改前只记在服务内存里，服务重启后同一档再报一次（会话 46 的任务 8819；task_server.py 的 note_attention_reported、
      stored_attention_tiers）
      「✗ task_server_lib.py 自证：attention-reported-survives-restart：」
  TASK_SERVER_BREAK=attention-queue-reasonless-sent  排队原因还没判出来的那一轮照改前照报排队过久、原因写读不到（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：attention-queue-waits-for-reason：」
  TASK_SERVER_BREAK=aged-seal-gap-filled  为满时限的整片封了组的那台照改前照样插空（task_server.py 的 Alternation.blocked）
      「✗ task_server_lib.py 自证：aged-seal-no-gap-fill：」
  TASK_SERVER_BREAK=budget-over-maxima-waits  记账超过每一台可用上限的照改前留在队里排着、不打回（task_server.py 的 budget_check、place_queued_into）
      「✗ task_server_lib.py 自证：budget-over-maxima-refused：」
  TASK_SERVER_BREAK=oversized-budget-silent  记账不小于整片界的不记成要处置的提醒 oversized-budget（task_server.py 的 budget_check、attention_of_record）
      「✗ task_server_lib.py 自证：oversized-budget-reported：」
  TASK_SERVER_BREAK=long-run-idle-silent  跑满 LONG_RUN_ATTENTION_SECONDS 而平均在用核数不到给的线程数四分之一的不记成要处置的提醒 long-run-idle-cores（task_server.py 的 long_run_idle_items）
      「✗ task_server_lib.py 自证：long-run-idle-cores-reported：」
  TASK_SERVER_BREAK=fills-any-cap-silent  同一命令键最近三次峰值都贴着各自上限的不记成要处置的提醒 fills-any-cap（task_server.py 的 fills_any_cap_items）
      「✗ task_server_lib.py 自证：fills-any-cap-reported：」
  TASK_SERVER_BREAK=fills-any-cap-counts-cap-hits  fills-any-cap 的那三次连撞了上限的尝试也算（照上限一档档撞上去的也报成按上限开并行；task_server.py 的 fills_any_cap_items）
      「✗ task_server_lib.py 自证：fills-any-cap-cap-hits-not-reported：」
  TASK_SERVER_BREAK=old-metric-peaks-read  定额度、编译族、兄弟片与判红峰值的下界照改前连没写峰值口径的旧尝试行一起读（task_server.py 的 current_metric）
      「✗ task_server_lib.py 自证：old-metric-peaks-not-counted：」
  TASK_SERVER_BREAK=attention-next-generic  anomaly_advice 不照异常表取，每一种都交一句通用的「看 status、等不起就取消改小重交」（提醒、打回的 next= 与发给会话的正文都退回它）
      「✗ task_server_lib.py 自证：anomaly-advice-in-texts：」、「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-overrun：」、「✗ task_server_lib.py 自证：attention-quiet-busy-log：」、「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」、「✗ task_server_lib.py 自证：gpu-busy-quiet-not-stalled：」、「✗ task_server_lib.py 自证：attention-card-replaced：」、「✗ task_server_lib.py 自证：attention-card-replaced-after-finish：」、「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：attention-shard-advance-failed：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：attention-reported-survives-restart：」、「✗ task_server_lib.py 自证：attention-queue-waits-for-reason：」、「✗ task_server_lib.py 自证：oversized-budget-reported：」、「✗ task_server_lib.py 自证：long-run-idle-cores-reported：」、「✗ task_server_lib.py 自证：fills-any-cap-reported：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：budget-over-maxima-refused：」、「✗ task_server_lib.py 自证：anomaly-repushed-with-history：」、「✗ task_server_lib.py 自证：unclaimed-outcome-repushed：」（头两格之后推的，没复跑：各格核 next= 以异常表那一行起头、或核那一句修改意见在不在）
  TASK_SERVER_BREAK=unclaimed-outcome-not-repushed  结局满 30 分钟没交给会话的不再推（task_server.py 的 repush_unclaimed_outcomes）
      「✗ task_server_lib.py 自证：unclaimed-outcome-repushed：」
  TASK_SERVER_BREAK=anomaly-not-repushed  不随时长涨的提醒同一档推过之后一直还在的不再推（task_server.py 的 attention_repush）
      「✗ task_server_lib.py 自证：anomaly-repushed-with-history：」
  TASK_SERVER_BREAK=inbox-socket-from-submit-only  发消息照交任务时记的收件 socket，不按会话号读会话登记（task_server.py 的 inbox_socket_of）
      「✗ task_server_lib.py 自证：inbox-socket-follows-session：」
  TASK_SERVER_BREAK=inbox-missing-accepted  客户端读不到会话的收件地址也照交（task 的 submit）
      「✗ task_server_lib.py 自证：inbox-unreachable-task-runs：」
  TASK_SERVER_BREAK=confirm-skipped  交任务不等确认就进队（task_server.py 的 submit）
      「✗ task_server_lib.py 自证：submit-awaits-confirmation：」、「✗ task_server_lib.py 自证：unconfirmed-cancelled：」、「✗ task_server_lib.py 自证：operation-log：」（后两格推的，没复跑：不等确认就跑完、操作日志里没有 confirm 那一条）
  TASK_SERVER_BREAK=confirm-counts-unchecked  task confirm 不核项数与在排在跑的条数（task_server.py 的 confirm）
      「✗ task_server_lib.py 自证：submit-awaits-confirmation：」
      「✗ task_server_lib.py 自证：requeue-refusals：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=unconfirmed-kept  交了没确认的满时限也不取消（task_server.py 的 expire_unconfirmed）
      「✗ task_server_lib.py 自证：unconfirmed-cancelled：」
  TASK_SERVER_BREAK=family-lends-run-peak  编译族借同族尝试的整趟峰值 memory_peak 与撞了上限那一次的上限，不只借编译段峰值（task_server.py 的 attempt_measurement）
      「✗ task_server_lib.py 自证：family-lends-build-phase-only：」
  TASK_SERVER_BREAK=seal-host-that-cannot-fit  整片挑等腾空的台时峰值本身就装不下它的台（记账超过那台此刻能给的最大上限 host_limit）也挑、为它封组，那台的碎片一直挡着（task_server.py 的 Alternation.__init__）
      「✗ task_server_lib.py 自证：whole-not-sealed-where-it-cannot-fit：」
  TASK_SERVER_BREAK=path-check-substring  提交时判共享根外的路径，每个带空白的词都按子串认，参数值中间夹着的 /（「（接的是 /）」）也当绝对路径、整条拒（task_server.py 的 find_outside_shared_path）
      「✗ task_server_lib.py 自证：path-check-whole-words：」
  TASK_SERVER_BREAK=build-phase-past-finished  本机采编译段峰值时见到 cargo 编完那一行之后照收，跑测试那一段的用量进编译段峰值（attempt_runner.py 文件头 ③）
      「✗ task_server_lib.py 自证：family-lends-build-phase-only：」
  TASK_SERVER_BREAK=cleanup-tests-tables-hand-listed  清场协议单测的假调度器照手抄的名单建表、漏 card_loads（tests/test_cleanup_protocol.py 的 scheduler）
      「✗ task_server_lib.py 自证：cleanup-protocol-unit-tests：」
  TASK_SERVER_BREAK=card-idle-stall-threshold-ignored  占着卡不干活（卡上有它的进程而利用率每一轮都是 0）的判卡住照不分卡的门槛 STALL_CPU_CORES，CPU 上轮询死等的照跑（task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：card-idle-polling-stalled：」
  TASK_SERVER_BREAK=peer-health-single-timeout-reported  连第二台的 ssh 头一次超时就算连不上、不复查（peer_shell_rechecked）
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」
  TASK_SERVER_BREAK=cap-hit-stat-unreported  内存包装打的撞上限那一刻的内存明细不抄进结局说明与推给会话的正文（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：cap-hit-stat-in-note：」
  TASK_SERVER_BREAK=over-two-hours-not-bounced  累计跑满两小时的不停（task_server.py 的 judge_run_limit）
      「✗ task_server_lib.py 自证：over-two-hours-bounced：」、「✗ task_server_lib.py 自证：over-two-hours-across-cap-hits：」、「✗ task_server_lib.py 自证：over-two-hours-check-not-exempt：」、
      「✗ task_server_lib.py 自证：over-two-hours-merge-and-verify-not-exempt：」、「✗ task_server_lib.py 自证：requeue-goes-to-tail：」、「✗ task_server_lib.py 自证：requeue-refusals：」、
      「✗ task_server_lib.py 自证：long-run-confirmed-not-bounced：」、「✗ task_server_lib.py 自证：whole-package-confirmed-still-bounced：」（后七格推的，没复跑：要一条 over-two-hours 的任务才往下判）
  TASK_SERVER_BREAK=over-two-hours-cap-hits-counted  撞了内存上限的那几次尝试的用时也累加，重来的那一次不归零（task_server.py 的 judge_run_limit）
      「✗ task_server_lib.py 自证：over-two-hours-across-cap-hits：」
  TASK_SERVER_BREAK=over-two-hours-commit-exempt-by-command-only  豁免只看起的是不是提交时的门禁，带 --check、「为什么」没写提交前的也豁免（commit_gate_exempt）
      「✗ task_server_lib.py 自证：over-two-hours-check-not-exempt：」
  TASK_SERVER_BREAK=requeue-keeps-original-position  task requeue 不新进队尾，把原号那一条照原任务号放回队里原来的位置（task_server.py 的 requeue）
      「✗ task_server_lib.py 自证：requeue-goes-to-tail：」
  TASK_SERVER_BREAK=long-run-confirmed-still-bounced  带「长跑已确认」标识的照样按两小时停（task_server.py 的 judge_run_limit）
      「✗ task_server_lib.py 自证：long-run-confirmed-not-bounced：」
  TASK_SERVER_BREAK=long-run-confirmed-flag-hidden  「长跑已确认」那一行不进 task status、列表、推给会话的正文与日志头（task_server.py 的 long_run_lines）
      「✗ task_server_lib.py 自证：long-run-confirmed-not-bounced：」
  TASK_SERVER_BREAK=whole-package-unconfirmed-accepted  整包测试没带确认标识也照收（task_server.py 的 whole_package_refusal）
      「✗ task_server_lib.py 自证：whole-package-unconfirmed-refused：」
  TASK_SERVER_BREAK=whole-package-confirmed-exempts-two-hours  带「整包已确认」标识的不按两小时打回（task_server.py 的 judge_run_limit）
      「✗ task_server_lib.py 自证：whole-package-confirmed-still-bounced：」
  TASK_SERVER_BREAK=whole-package-flag-dropped-on-requeue  task requeue 新交的那一条不带原号的「整包已确认」标识（task_server.py 的 requeue）
      「✗ task_server_lib.py 自证：whole-package-confirmed-still-bounced：」
自证另有几格不配开关：selftest-variables-refused（判的守卫在读开关之前，开关在别处设了本身就被它拒）；type-table-covered（静态核错误类型表里每一种都有格喂到）；control-watch-removed（control watch 只剩退 2 打出路句这一支，没有能弄坏的判法）；
anomaly-table-matches-kinds（静态核文件头异常表的种类与 ATTENTION_KINDS、OUTCOME_TYPES、SUBMIT_REFUSAL_KINDS 逐行对得上）。
"""
import os
import sys

sys.dont_write_bytecode = True

import calendar  # noqa: E402
import contextlib  # noqa: E402
import hashlib  # noqa: E402
import hmac  # noqa: E402
import http.client  # noqa: E402
import importlib.util  # noqa: E402
import json  # noqa: E402
import math  # noqa: E402
import re  # noqa: E402
import secrets  # noqa: E402
import shlex  # noqa: E402
import socket  # noqa: E402
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
GPU_QUIESCE = os.path.join(REPOSITORY_ROOT, "scripts", "crash", "gpu-quiesce-run.sh")
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
ADMITTED_VARIABLE = "RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER"   # 内存包装只认服务起的那一层的准入标记（run-with-memory-cap.sh 文件头「只认服务直接起的那一层」）
THREAD_ADMITTED_VARIABLE = "CAPPED_ADMITTED_BY_TASK_SERVER"   # capped.sh 只认服务套的那一层（capped.sh 文件头；task_server.py 文件头 ③「起」）：两台都设
QUIESCE_ADMITTED_VARIABLE = "GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER"   # 清场包装只认服务套的那一层（gpu-quiesce-run.sh 文件头）：套了清场时设
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
# 十二种提醒（task_server.py 的 attention_items 判，文件头「需要处置」）：排队过久、跑到记录用时的很多倍、日志长期不涨而服务没停它、卡在 IO 上而服务没停它、
# 卡上有它的计算进程而服务没停它、换卡重排、清场没收尾、分片父任务推不动、改主工作区的命令没正常结束、预算贴着整台、跑得久而核没用起来、按给的上限开并行
ATTENTION_KINDS = ("queued-too-long", "overrun", "quiet-log-busy", "io-wait", "gpu-busy-quiet", "card-replaced", "cleanup-pending", "shard-advance-failed",
                   "workspace-write-interrupted", "oversized-budget", "long-run-idle-cores", "fills-any-cap")
# 异常表（文件头「异常表」）：那几栏只写在文件头那一段，anomaly_table 照 __doc__ 现读，代码里不另抄一份；下面几组是代码里分别处置的种类，自证 anomaly-table-matches-kinds 拿它们与表比
ANOMALY_ACTIONS = ("提交时拒", "打回", "提醒照跑", "服务自己修")
ANOMALY_TABLE_HEAD = "异常表（"
ANOMALY_ROW = re.compile(r"^  ([a-z][a-z-]*)｜([^｜]+)｜([^｜]+)｜([^｜]+)｜([^｜]+)$")
SUBMIT_REFUSAL_KINDS = ("environment-refused", "nested-wrapper", "target-directory-in-repository", "outside-shared-path", "shared-roots-outside",
                        "verdict-library-unserved", "temporary-service-heavy", "whole-workspace-cargo", "inbox-missing", "confirm-counts-mismatch",
                        "requeue-counts-mismatch", "whole-package-unconfirmed")
SUBMIT_WARNING_KINDS = ("repository-target-bypass", "unclaimed-results")
SERVICE_ANOMALY_KINDS = ("unconfirmed", "suspected-stall", "unclaimed-outcome")
# 只给弄坏开关 attention-next-generic：每一种都退回这一句通用的出路
ANOMALY_GENERIC_ADVICE = "task-server/task status {task} 看在等什么，等不起 task-server/task cancel {task}、改小命令重交"
ANOMALY_TASK_PLACEHOLDER = "{task}"
ANOMALY_TABLE_LOADED = []   # 读过的那一份（anomaly_table）
# 交任务的二次确认（task_server.py 文件头 ⑦）：交了的任务先在这个状态、不进队，task confirm 两个数对得上才进队
AWAITING_CONFIRMATION_STATE = "awaiting-confirmation"
CONFIRM_NEEDED_MARK = "TASK_SERVER confirm-needed"   # 客户端打确认单之后那一行（机器读的：自证的帮手照它确认）
CONFIRM_NEEDED_LINE = re.compile(r"^TASK_SERVER confirm-needed task=([0-9]+) items=([0-9]+) tasks=([0-9]+)(?: |$)", re.MULTILINE)
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
REQUEUE_MARK = "TASK_SERVER requeue"   # task requeue 收下时 stdout 那一行的开头（task 文件头 requeue）：后面接 task=<新号> from=<原号>
CARD_FAULT_MARK = "GPU_QUIESCE step=quiesce result=card-fault"
CARD_FAULT_LIST_FIELD = "faulty-cards="   # 清场 card-fault 那一行里的坏卡清单（文件头「错误类型」card-fault 那一行）
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
NFS_SSH_SECONDS = 30
NFS_RECHECK_PAUSE_SECONDS = 5.0   # 连第二台的 ssh 一次超时之后隔这么久复查（文件头「NFS 共享」的挂载）：第二台负载高时一次慢不当连不上，只复查一次
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
    "memory-cap-at-maximum": ("host", "reached", "peak"), "squeezed-out": ("host", "cap", "by"), "gpu-memory-short": ("card", "quiesce"),
    "timed-out": ("cap", "last_line"), "wrapper-refused": ("last_line",), "run-condition-unmet": ("card", "quiesce", "last_line"), "not-run": ("last_line",),
    "peer-run-failed": ("host", "last_line"), "prune-server-unavailable": ("host", "last_line"), "card-fault": ("card", "quiesce"), "launcher-lost": ("unit",),
    "cancelled": ("by",), "record-tampered": ("what",),
    "never-fits": ("judged",), "unit-start-failed": ("unit", "judged"), "resubmit-required": ("rule",), "stalled": ("judged",), "shard-plan-refused": ("judged",),
    "over-two-hours": ("host", "last_line"),
}
# 内存包装判出撞上限（退 250）那一刻打的一行（scripts/hosts/run-with-memory-cap.sh 文件头「撞上限那一刻的内存明细」；两台同一行，第二台的经 multi-host-run.sh 接回）：
# 服务把它换成一句抄进那一次尝试的结局说明（oom_stat_note）。两边的格式一起改
OOM_STAT_LINE = re.compile(r"^RUN_WITH_MEMORY_CAP_OOM_STAT anon=(\d+|unread) file=(\d+|unread) file_dirty=(\d+|unread) file_writeback=(\d+|unread) shmem=(\d+|unread) oom_kill=(\d+|unread)$")
# 跑满两小时打回的豁免（文件头「错误类型」over-two-hours）：提交时的六道门禁与提交前整轮（仓根下的相对路径），范围声明的「为什么」里要有 COMMIT_SCOPE_WORD
COMMIT_GATE_ENTRIES = (".claude/gate.d/crash-point-replay.sh", ".claude/gate.d/qemu-device-streams.sh", ".claude/gate.d/lkmm.sh",
                       ".claude/gate.d/crates-mutation-replay.sh", ".claude/gate.d/formal-verification.sh", ".claude/gate.d/broad-mutation.sh",
                       "scripts/gate/gate-staged.sh")
COMMIT_SCOPE_WORD = "提交前"
SCOPE_RANGE_ARGUMENTS = ("--check", "--item")   # 带了就是指定范围，不是提交时那一趟
# 整包测试（task_server.py 文件头 ②「提交时拒」⑥；.claude/agent-common.md「日常开发的测试不跑整个包」）：crash-point-replay.sh --check 把这三格全点了算整包
WHOLE_PACKAGE_CHECK_CELLS = ("crash-case-replay", "feature-gated-tests", "product-gpu-real-card-tests")
WHOLE_PACKAGE_CRASH_GATE = ".claude/gate.d/crash-point-replay.sh"
WHOLE_PACKAGE_MERGE_ENTRY = "scripts/gate/merge-and-verify.sh"
MERGE_VALUE_OPTIONS = ("--crash-expand-up-to", "--item")   # merge-and-verify.sh 带值的选项（它文件头的用法行）：值不当补丁目录
# cargo 子命令后面带值的选项：值不当测试名（whole_package_cargo_test 认测试名用）
CARGO_VALUE_OPTIONS = ("-p", "--package", "-F", "--features", "--profile", "--target", "--test", "--bin", "--example", "--bench", "-j", "--jobs",
                       "--manifest-path", "--target-dir", "--color", "--message-format", "-Z", "--config")
WHOLE_PACKAGE_MARKS = ("①", "②", "③")   # 整包确认文件的三段（.claude/agent-common.md「日常开发的测试不跑整个包」的三遍确认）各以它起头
WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS = 12   # 每段去掉空白（与段首的记号）至少这么多个字
WHOLE_PACKAGE_NEXT_STEP = ("改成只点名改到的测试目标与测试函数；确实要整包的照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍，"
                           "带 --whole-package-confirmed <确认文件> 重交（task-server/task submit --whole-package-confirmed <确认文件> -- <命令…>）")
# multi-host-run.sh 打进日志的几行（文件头「错误类型」与「资源记录」）：第二台那条命令自己的退出码、第二台那一层的内存峰值与编译目录占盘峰值
PEER_COMMAND_END_LINE = re.compile(r"^MULTI_HOST_RUN_STEP step=peer-command event=end(?: \S+)* exit=([0-9]+)(?: |$)")
PEER_COMMAND_HOST = "peer-command"   # classify_attempt 的 host：第二台那条命令自己的退出码（255 也是 command-failed）
PEER_MEMORY_PEAK_MARK = "MULTI_HOST_RUN_PEER_MEMORY_PEAK"
PEER_BUILD_PEAK_MARK = "MULTI_HOST_RUN_PEER_BUILD_PEAK"
PEER_BUILD_PHASE_PEAK_MARK = "MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK"   # multi-host-run.sh 打的第二台那条命令的编译段峰值（文件头「资源记录」的编译族；它文件头 ⑤「编译段峰值」）
# cargo 编完打的那一行（「    Finished `release` profile [optimized] target(s) in 1m 02s」）：它之前按轮采到的是编译段（文件头「资源记录」的编译族）；
# 交给 attempt_runner.py 的说明 build_finished_pattern（字节串照 utf-8 编），multi-host-run.sh 的 MULTI_HOST_CARGO_FINISHED_PATTERN 是同一个式子的 ERE 写法，两边一起改
CARGO_FINISHED_PATTERN = r"^\s*Finished\b.*\btarget\(s\) in "
PEER_BUILD_KEPT_MARK = "MULTI_HOST_RUN_PEER_BUILD_KEPT"   # multi-host-run.sh 带 --keep-build-directory 跑完留下这一趟的编译目录时打「<它> label=<标签>」（它文件头「留着的编译目录」）
REASON_LOG_NOISE = ("MULTI_HOST_RUN_STEP ", "MULTI_HOST_RUN_PEER_CPU ", "MULTI_HOST_RUN_PEER_IO_WAIT ", PEER_MEMORY_PEAK_MARK + " ", PEER_BUILD_PEAK_MARK + " ", PEER_BUILD_PHASE_PEAK_MARK + " ", PEER_BUILD_KEPT_MARK + " ",
                    INBOX_LOG_MARK + " ")   # 取日志最后一行时跳过的进度行（task_server.py 判日志涨不涨同样不算它们）
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
GPU_QUIESCE_LINE_MARK = "GPU_QUIESCE "

# 分片（文件头「分片」）
SHARDED_ENTRIES = {   # 仓根下的相对路径 → 第一个参数认哪几个（None：除 NOT_SHARDED_ARGUMENTS 之外都分）
    "scripts/crash/crash-point-replay-shard-run.sh": ("--merged-log",),
    "scripts/mutation/mutation-shard-run.sh": None,
    "scripts/crash/crash-amplification-two-hosts.sh": None,
    "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py": ("run",),
}
NOT_SHARDED_ARGUMENTS = ("--selftest", "--list-items", "--help", "-h")
DECLARED_SHARDS_MARK = "# task-server-shards:"
DECLARED_HEADER_LINES = 60   # 脚本文件头里的声明（分片、要卡的参数、续跑）只读前这么多行
# 要卡的参数（文件头「GPU」）：命令位置上起的脚本文件头写「# task-server-gpu-arguments: <词> [<词>…]」，第一个非选项参数是其中一个就要卡（* 是任何参数）
DECLARED_GPU_ARGUMENTS_MARK = "# task-server-gpu-arguments:"
# 要几张卡（文件头「GPU」的张数）：脚本文件头写「# task-server-gpu-cards: <张数>」，Rust 源文件头写「// task-server-gpu-cards: <张数>」；没写是一张
DECLARED_GPU_CARDS_MARK = "# task-server-gpu-cards:"
DECLARED_SOURCE_GPU_CARDS_MARK = "// task-server-gpu-cards:"
# 编译族（文件头「资源记录」的编译族）：入口脚本文件头写「# task-server-cargo-family: <cargo 参数>…」，说它里面起的 cargo 编哪个包、哪一档、哪几个特性
DECLARED_BUILD_FAMILY_MARK = "# task-server-cargo-family:"
FAMILY_ARGUMENT_MARK = "$"   # 编译族声明里取自命令参数的词以它起头：$<n> 取第 n 个参数，$<选项名> 取那个选项的值（filled_family_words）
BUILD_FAMILY_MARK = "〈编译族〉"   # 编译族那一段文本的头（cargo_build_family 交的）
PACKAGE_SKIPPED_DIRECTORIES = ("target", ".git", "node_modules", "src", "tests", "benches", "examples")   # 找包目录（package_directories）时不往下走的目录
# 续跑（文件头「续跑」）：命令位置上起的脚本文件头写「# task-server-resumable: <词> [<词>…]」，第一个非选项参数是其中一个，就算这条命令登记了续跑（* 是任何参数）
DECLARED_RESUMABLE_MARK = "# task-server-resumable:"
LONG_RUN_SECONDS = 30 * 60   # 用户 2026-09-26：跑超过 30 分钟的要能中断续跑；跑这么久而失败、又没登记续跑的，结局说明写一句重交会从头跑
MAXIMUM_SHARDS = 64   # 按项分片的入口一项一片（崩溃放量全量 29 项，scripts/crash/crash-amplification.sh 文件头「经 task-server 按项分片」）
SHARD_PHASE_VARIABLE = "TASK_SERVER_SHARD_PHASE"
SHARD_VARIABLE = "TASK_SERVER_SHARD"
SHARD_PLAN_VARIABLE = "TASK_SERVER_SHARD_PLAN_DIRECTORY"
SHARD_OUTPUT_VARIABLE = "TASK_SERVER_SHARD_OUTPUT_DIRECTORY"
SHARD_OUTPUTS_VARIABLE = "TASK_SERVER_SHARD_OUTPUTS"
SHARD_HOST_VARIABLE = "TASK_SERVER_SHARD_HOST"
SHARD_MISSING_VARIABLE = "TASK_SERVER_SHARD_MISSING"   # 合并那一步：人手单独取消、没有产物的那几片的片号（文件头「分片」③）
SHARDS_ROOT_NAME = "task-server-shards"   # 工作根下放分片计划与各片产物的目录（文件头「分片」：两台照同一个绝对路径读写）
SCENES_ROOT_NAME = "task-server-scenes"   # 工作根下放判红那一趟现场的目录，每一次尝试一个 <尝试标签>（文件头「现场」）
SCENE_VARIABLE = "RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY"   # 交给内存包装的这一次尝试的现场目录（文件头「现场」）
PEAK_OUTPUT_VARIABLE = "RUN_WITH_MEMORY_CAP_PEAK_OUTPUT"   # 交给内存包装的退出时峰值那一份（文件头「资源记录」的退出时峰值）
EXIT_PEAK_FILE_NAME = "exit_memory_peak"   # 尝试目录里内存包装写退出时峰值的那一份
BUILD_PHASE_FILE_NAME = "build_phase"   # 尝试目录里 attempt_runner 写编译段峰值与见没见到 cargo 编完那一行的那一份（文件头「资源记录」的编译族；自证读它）
MEMORY_PEAK_FILE_NAME = "memory_peak"   # 尝试目录里 attempt_runner 每轮写进程内存见过的最大的那一份（文件头「资源记录」的峰值的口径；task_server.py 的 scope_memory_peak 读它）
PLAN_FILE = "plan.json"
SHARD_PLAN_KEYS = ("label",)
SHARD_ROLES = ("plan", "shard", "merge")   # 分片那三步的子任务（文件头「分片」）；估算那一步（ESTIMATE_ROLE）不算

# 按项估预计用时（文件头「资源记录」的预计用时）
ITEM_ENTRIES = ("scripts/crash/crash-point-replay.sh", ".claude/gate.d/formal-verification.sh",
                ".claude/gate.d/crates-mutation-replay.sh", ".claude/gate.d/crash-point-replay.sh",
                "scripts/gate/merge-and-verify.sh")   # 仓根下的相对路径：按项判、打 ITEM_PLAN 与 ITEM_DONE 的五条入口（后两条：文件头写了这两行，merge-and-verify 的列项任务 7587 打出 373 行 ITEM_PLAN）
ITEM_ARGUMENT = "--item"
ITEM_ESTIMATE_ARGUMENTS = ("--list-items", "--with-keys")   # 估算子任务接在入口后面的两个参数（列每项要现判与能复用的条数）
ESTIMATE_ROLE = "estimate"   # 估算子任务的 role
ESTIMATE_KEY_MARK = "〈task-server 估算〉"   # 估算子任务的命令键原文接在父任务的键后面：与父任务不共用峰值与用时记录
ITEM_PLAN_LINE = re.compile(r"^ITEM_PLAN item=(\S+) to_judge=([0-9]+) reusable=([0-9]+)$")
ITEM_DONE_LINE = re.compile(r"^ITEM_DONE item=(\S+) seconds=([0-9]+(?:\.[0-9]+)?) judged=([0-9]+) reused=([0-9]+) exit=(-?[0-9]+)$")
ITEM_LINE_PREFIX = b"ITEM_"
ITEM_RATE_STATE = "item-rate"   # item_rates 表里每一行的 state
ESTIMATE_BASIS_COMMAND = "command"   # 服务回复里 estimate_basis 的三个取值（文件头「资源记录」的预计用时）：照这条命令最近一次跑成的用时
ESTIMATE_BASIS_ITEMS = "items"       # 照点到的每一项的现判条数、复用条数和两个速率相加
ESTIMATE_BASIS_NONE = "none"         # 没有预计用时
ESTIMATE_BASIS_SENTENCES = {         # 客户端 task status <号> 在预计用时后面打的说明，一种算法一句
    ESTIMATE_BASIS_COMMAND: "（照这条命令最近一次真跑完的用时）",
    ESTIMATE_BASIS_ITEMS: "（照点到的每一项：要现判的条数 × 每现判一条几秒 + 能复用的条数 × 每复用一条几秒，相加）",
    ESTIMATE_BASIS_NONE: "（没有可用的记录：这条命令没有真跑完过的记录，点了 --item 的是点到的项缺记录）"}

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
    QUIESCE_ADMITTED_VARIABLE: "这个变量只由 task-server 设给它套的那一层 gpu-quiesce-run.sh",
    PEER_ADMITTED_VARIABLE: "这个变量只由 task-server 设给派第二台的那一层 multi-host-run.sh --peer",
    "RUN_WITH_MEMORY_CAP_OUTER_UNIT": "提交客户端是在别的内存包装的 scope 里起的：不在内存包装里面提交，在外面提交",
    "RUN_WITH_MEMORY_CAP_OUTER_CAP": "提交客户端是在别的内存包装的 scope 里起的：不在内存包装里面提交，在外面提交",
    "GPU_CARDS_BY_PRIORITY": "分卡归 task-server：照多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 挑",
    "CUDA_VISIBLE_DEVICES": "分卡归 task-server：照多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 挑",
}
TAKEN_OVER_PREFIXES = ("RUN_WITH_MEMORY_CAP_", "HOST_DISPATCH_")
# 退役的名字（文件头「环境变量」）：名字 → 为什么不收与出路；环境里设了、命令外层写了「名=值」都当场拒
RETIRED_VARIABLES = {
    "SINGLEFS_HEAVY_TESTS": "退役了：交给服务的命令不写这个前缀，重型测试照 .claude/rules/verification.md「重型：三种跑法」由主 agent 在提交时或用户要求时交：unset 它，命令里也不写",
    "GATE_MUTATION_START_OVER": "已删：crates 变异门禁全量只认 SINGLEFS_GATE_FULL=1：unset 它，要全量写 -- env SINGLEFS_GATE_FULL=1 <命令…>",
}
NESTED_DISPATCH_SCRIPTS = ("run-with-memory-cap.sh", "host-dispatch.py", "multi-host-run.sh")
# 用户定 M06：提交的命令不带包装前缀与脚本自己的上限、线程、挑台参数，由服务一处拒（records/2026-10-30-编译绕过内存总上限.md）
REFUSED_PREFIXES = {"nice": "优先级", "timeout": "限时", "capped.sh": "线程", "two-logs.sh": "日志", "gpu-quiesce-run.sh": "清场与挑卡"}
RESOURCE_ARGUMENTS = ("--memory", "--memory-cap", "--threads", "--peer", "--local-only")
WRAPPER_OWN_ARGUMENTS = (["--selftest"], ["--list-items"], ["--help"], ["-h"])

# 错误类型：名字 → (wait 的退出码，None 是照命令自己的；说明)。文件头「错误类型」是同一张表
OUTCOME_TYPES = {
    "succeeded": (0, "命令退 0"),
    "command-failed": (None, "命令自己退非 0（表外的退出码都在这里）"),
    "memory-cap-hit": (250, "撞了这一条自己的上限"),
    "memory-cap-at-maximum": (236, "撞上限、已到两台最大"),
    "squeezed-out": (254, "被总上限挤掉"),
    "gpu-memory-short": (245, "显存不够"),
    "timed-out": (253, "超时"),
    "wrapper-refused": (251, "内存包装用法错或 scope 起不来"),
    "run-condition-unmet": (78, "运行条件不满足"),
    "not-run": (77, "本次未跑"),
    "peer-run-failed": (255, "第二台没跑成"),
    "prune-server-unavailable": (75, "prune-server 不可用（命令自己退 75）"),
    "card-fault": (237, "清场判出卡坏"),
    "launcher-lost": (238, "对不回来"),
    "cancelled": (239, "被取消"),
    "record-tampered": (240, "库被改过（校验不过）"),
    "never-fits": (241, "哪台都等多久也放不下"),
    "unit-start-failed": (235, "尝试的单元起不来"),
    "shard-plan-refused": (234, "分片的计划读不通"),
    "over-two-hours": (242, "不是提交时的门禁，累计跑满两小时，服务停了它"),
    "stalled": (233, "日志满 10 分钟没涨，服务停了它"),
    "resubmit-required": (232, "服务规则改了，照现行规则不再收"),
}
EXIT_TYPES = {0: "succeeded", 75: "prune-server-unavailable", 77: "not-run", 78: "run-condition-unmet", 245: "gpu-memory-short", 250: "memory-cap-hit",
              251: "wrapper-refused", 253: "timed-out", 254: "squeezed-out"}
PRUNE_SERVER_UNAVAILABLE_OUTCOME = "prune-server-unavailable"   # 75：scripts/prune-server/prune_server_client.py 的 EXIT_UNAVAILABLE
PRUNE_SERVER_ALERT_KIND = "prune-server-unavailable"            # 命令退 75 而 prune-server/control check 不通时写的告警
CAP_GROWTH_FACTOR = 1.5          # 用户定：撞上限就把额度乘 1.5 重排（records/2026-10-30-编译绕过内存总上限.md）；定额度照记录也乘它
MEMORY_CAP_RETRIES = 16
SQUEEZE_RETRIES = 3
GPU_SHORT_RETRIES = 1
CARD_FAULT_RETRIES = 4           # 清场判出卡坏不是任务的错：那张卡停用（task_server.py 的 CARD_QUARANTINE_SECONDS），同一上限换别的卡重排
PEER_RUN_RETRIES = 2             # 第二台没跑成（连不上、ssh 被重置、那边的准备没过）不是任务的错：同一上限重排、不钉台
RETRY_LIMITS = {"memory-cap-hit": MEMORY_CAP_RETRIES, "squeezed-out": SQUEEZE_RETRIES, "gpu-memory-short": GPU_SHORT_RETRIES, "peer-run-failed": PEER_RUN_RETRIES,
                "card-fault": CARD_FAULT_RETRIES}
MEBIBYTE = 1024 * 1024
GIBIBYTE = 1024 * MEBIBYTE
# 没有记录的命令的预测、上限与记账（文件头「资源记录」）：用户选「没记录的先记小，撞了再放大」，主 agent 定 4G
# （records/2026-10-30-编译绕过内存总上限.md「问题清单的去向」#6）：任务 1–8 实测峰值 10 MiB 到 1.71 GiB；常见命令已从峰值表导入 3321 条记录
UNRECORDED_MEMORY_DEFAULT_BYTES = 4 * GIBIBYTE
MEMORY_FLOOR_BYTES = UNRECORDED_MEMORY_DEFAULT_BYTES   # 改前的 4G 底（文件头「资源记录」的有记录的照记录）：只在弄坏开关 recorded-cap-floored 下兜
MEMORY_FLOOR_VARIABLE = "TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES"   # 只在自证里认：各格的小世界换这个下限（文件头「只在自证里认的变量」）
UNRECORDED_HISTORY = "unrecorded-default"   # 尝试记录 prediction 一栏：这一次是按没记录的小缺省定的
RECORD_HISTORY = "record"                   # 尝试记录 prediction 一栏：这一次的上限照服务自己记下的峰值 × 1.5 定（文件头「资源记录」的定额度）
FAILED_PEAK_HISTORY = "failed-peak"         # 尝试记录 prediction 一栏：这一次的上限照判红那一次量到的峰值 × 1.5 定（文件头「资源记录」）
BUILD_FAMILY_HISTORY = "build-family"       # 尝试记录 prediction 一栏：照同一编译族别的命令近 RECORD_WINDOW 次的编译段峰值的最大 × 1.5 起步（文件头「资源记录」的编译族）
SHARD_FAMILY_HISTORY = "shard-family"       # 尝试记录 prediction 一栏：分片的一片照同一计划里计划那一步与兄弟片的峰值往上抬过（文件头「资源记录」的同一计划共用峰值）
KEY_HISTORY_PEAK_HISTORY = "key-history"    # 尝试记录 prediction 一栏：照这把键历史上最高的一次（撞过的上限、跑成的峰值）× 1.5 往上抬过（文件头「资源记录」的按历史最高起跑）
WORKSPACE_WRITER_HISTORY = "workspace-writer"   # 尝试记录 prediction 一栏：改主工作区的命令首趟照这把键与它的验证命令近几次最高之和 × 1.5 往上抬过（文件头「资源记录」）
PROCESS_MEMORY_PEAK_METRIC = "进程内存"   # 尝试记录 peak_metric 一栏：memory_peak 是进程用的内存（anon 加 shmem；文件头「资源记录」的峰值口径一栏）；没有这一栏的旧行不读
# 命令键换算之前剥掉的环境变量前缀（文件头「资源记录」的命令键）：这几个只改配置路径、范围声明、语言这类，不改命令要多少内存
KEY_NEUTRAL_VARIABLES = ("SINGLEFS_MULTI_HOST_CONFIG", "SINGLEFS_RUN_SCOPE", "BINDGEN_EXTRA_CLANG_ARGS", "LANG")
ASSIGNMENT_WORD = re.compile(r"[A-Za-z_][A-Za-z0-9_]*=", re.DOTALL)
# 定额度只用跑完的尝试（文件头「资源记录」）：退 0 的记量到的峰值，撞了这一条自己上限的记「至少要到这个上限」；别的结局的峰值偏小或不作数，不进
RECORD_COMPLETED_OUTCOMES = ("succeeded",)
# 预计用时取的尝试（文件头「资源记录」的预计用时）：命令自己跑到退出的两种结局；被信号杀的 command-failed 由 task_server.py 的 command_estimate 另排掉
ESTIMATE_COMPLETED_OUTCOMES = ("succeeded", "command-failed")
RECORD_CAP_HIT_OUTCOMES = ("memory-cap-hit", "memory-cap-at-maximum")
# Linux 的 sockaddr_un.sun_path 是 108 字节；留一个结尾的零，路径本身至多 107 字节（文件头「路径」）
SOCKET_PATH_MAXIMUM_BYTES = 107

SELFTEST_MARK_VARIABLE = "TASK_SERVER_SELFTEST_MARK"
SELFTEST_MARK_TEXT = "task-server-selftest"
BREAK_VARIABLE = "TASK_SERVER_BREAK"
SELFTEST_ONLY_VARIABLES = (
    BREAK_VARIABLE, "TASK_SERVER_SELFTEST_FAST", "TASK_SERVER_SELFTEST_WORLD", "TASK_SERVER_FAKE_RESOURCES", "TASK_SERVER_MEMORY_WRAPPER",
    "TASK_SERVER_PEER_RUNNER", "TASK_SERVER_GPU_QUIESCE", "TASK_SERVER_SYSTEMD_RUN", "TASK_SERVER_CONTROL_SYSTEMCTL", "TASK_SERVER_NVIDIA_SMI",
    "TASK_SERVER_FAKE_SCOPE_DIRECTORY", "TASK_SERVER_FAKE_REPOSITORY_ROOT", "TASK_SERVER_SESSION_BOUNDARY_PID", "TASK_SERVER_SELFTEST_MISSING_MODULES",
    "TASK_SERVER_SELFTEST_CRASH_POINT", "TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE", "TASK_SERVER_SELFTEST_ALLOW_TMPFS", "TASK_SERVER_SELFTEST_SOCKET_ROOT",
    "TASK_SERVER_SELFTEST_AGED_SECONDS", "TASK_SERVER_SELFTEST_PLACEMENT_PAUSED", "TASK_SERVER_SELFTEST_PLACEMENT_REFUSED", "TASK_SERVER_SELFTEST_STALL_SECONDS", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS",
    "TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS", "TASK_SERVER_NFS_EXPORTS", "TASK_SERVER_PEER_SSH", "TASK_SERVER_SELFTEST_PAUSE_POINT",
    "TASK_SERVER_SELFTEST_STOP_FAILS", "TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS", "TASK_SERVER_PRUNE_SERVER_CHECK", MEMORY_FLOOR_VARIABLE,
    "TASK_SERVER_SELFTEST_LONG_RUN_SECONDS", "TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS", "TASK_SERVER_SELFTEST_OVERRUN_ATTENTION_FACTOR",
    "TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS", "TASK_SERVER_SELFTEST_ADVANCE_FAILS", "TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS",
    "TASK_SERVER_SELFTEST_PROC_ROOT", "TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS", "TASK_SERVER_SELFTEST_GPU_BUSY_QUIET_ATTENTION_SECONDS",
    "TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS", "TASK_SERVER_SELFTEST_SESSIONS_DIRECTORY", "TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS",
    "TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS", "TASK_SERVER_SELFTEST_NFS_SSH_SECONDS", "TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS",
    "TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS")
KNOWN_BREAK_ITEMS = (
    "fallback-run-when-unreachable", "exit-code-dropped", "wait-gives-up-on-disconnect", "environment-unfiltered", "queue-variables-carried",
    "retired-variables-carried", "retired-variable-assignments-accepted", "nested-dispatch-allowed", "session-from-client", "cancel-session-unchecked", "cancel-no-readback", "signature-unchecked", "tamper-unalerted",
    "operations-unlogged", "unlisted-exit-swallowed", "card-fault-unread", "card-fault-unalerted", "peer-exit-unsplit", "lost-not-recorded",
    "cap-growth-skipped", "cap-unclamped", "host-switch-skipped", "retry-overwrites-attempts", "squeeze-retry-skipped", "gpu-short-retry-skipped",
    "server-pending-ignored", "launch-when-unplaceable", "queue-reversed", "nofile-limit-dropped", "waiting-reason-unreported", "estimate-only-succeeded", "backfill-disabled", "gpu-unrecognized", "gpu-entry-scripts-unrecognized",
    "card-priority-ignored", "card-labels-all-local", "card-busy-ignored", "card-wait-blocks-all", "no-detach", "start-failure-ignored", "start-twice-after-restart",
    "resources-unrecorded", "record-ignored-for-cap", "unclaimed-not-listed", "check-deps-skipped", "config-missing-ignored",
    "state-placement-unchecked", "dry-run-writes", "secrets-overwritten", "install-readback-skipped", "stop-exit-status-only",
    "target-directory-unset", "retry-build-directory-kept", "clean-removes-nothing", "clean-session-unchecked", "runner-wake-skipped", "wait-notify-skipped", "aged-task-not-prioritized", "starved-head-unreported", "blocked-head-counted-as-starved", "in-repository-target-accepted",
    "unmeasured-recorded-as-zero", "estimate-unreported", "threads-unset", "maximum-line-vague", "sharding-unrecognized",
    "shard-retry-restarts-all", "merge-before-all-green", "shard-failure-unnamed", "balance-ignored", "retry-pinned-to-host", "stall-ignored", "stall-grace-skipped", "cpu-unread-counted-as-idle", "outer-scope-cpu-only", "inner-commands-unchecked", "queued-not-revalidated", "cpu-ignored-for-stall", "overrun-not-stopped", "peer-run-retry-skipped", "peer-starts-unthrottled", "squeeze-at-total-not-cap-hit", "prove-red-gpu-unread", "entry-quiesce-skipped", "host-specific-variables-sent-to-peer", "running-progress-unreported", "wrapper-prefixes-allowed", "resource-arguments-allowed", "peer-overflow-blocked",
    "shard-not-sent-to-peer", "shard-cap-not-fitted", "shard-key-position-folded", "socket-path-unchecked", "unrecorded-default-grown", "unrecorded-booked-above-cap",
    "unfinished-peaks-counted", "cap-hit-not-counted",
    "completed-peak-ignored", "log-tail-unbounded", "log-all-ignored", "log-attempt-ignored", "log-missing-exit-zero", "queued-cancel-left-queued",
    "finished-cancel-overwrites", "sharded-cancel-children-left", "plan-failure-proceeds", "merge-failure-succeeds", "unsharded-arguments-sharded",
    "start-readback-skipped", "start-exit-status-ignored", "cardless-gpu-launched", "pinned-peer-unavailable-waits",
    "prediction-recomputed-every-pass", "pending-read-per-task", "pass-booking-not-carried", "launch-recheck-skipped",
    "gpu-feature-any-subcommand", "no-run-needs-card", "mutation-gate-gpu-unread",
    "stall-clock-reset-on-restart", "workspace-writers-retried", "judge-error-blocks-queue", "stalled-at-cap-not-retried", "booked-up-to-slice-total",
    "memory-cap-not-passed", "source-repository-not-passed", "suspect-unreported", "suspect-card-load-ignored", "suspect-reported-every-wait", "pinned-cap-ignored", "pinned-cap-grown",
    "pinned-cap-literal", "pin-unreadable-predicted", "booked-at-peak", "health-judges-head", "python-entry-gpu-unrecognized",
    "alternation-disabled", "whole-waits-for-empty-host", "sealed-group-held-when-drained", "pinned-cap-at-slice-total", "margin-fallback-fixed", "margin-unmeasured-waits",
    "margin-local-for-both", "pinned-nonpositive-accepted", "ended-attempts-unsettled", "feed-only-counted-as-writer", "registry-service-column-ignored", "cpu-gate-ignored", "unstarted-slice-estimated", "down-host-judged-never", "card-fault-not-replaced", "failed-peak-ignored",
    "environment-prefix-kept", "nfs-unchecked", "nfs-probe-one-way", "nfs-probe-at-root", "nfs-probe-shared-name", "finished-build-kept", "quoted-word-paths-ignored", "expansion-tail-as-path",
    "peer-unavailable-silent", "reason-line-omitted", "clean-failure-unexplained",
    "shared-roots-not-passed", "peer-admission-unmarked", "disk-floor-unchecked", "peer-tree-copied", "workspace-writers-local-only",
    "cancel-race-unguarded", "sharded-parents-unreconciled", "attempt-claim-split", "sharded-cancel-finished-first",
    "association-column-unchecked", "start-timeout-as-failure", "build-disk-unpredicted", "peer-step-exit-ignored", "nfs-mount-placeholder-accepted",
    "peer-memory-peak-ignored", "prune-server-exit-unsplit", "temporary-service-wrapper-kept", "temporary-service-key-shared",
    "host-lists-restored", "data-root-key-unchecked", "outside-path-accepted",
    "shared-roots-outside-accepted", "verdict-library-family-accepted", "temporary-service-heavy-accepted",
    "pinned-cap-this-host-only", "by-host-missing-accepted", "by-host-outside-accepted", "shard-host-role-passed",
    "gpu-feature-names-unread", "gpu-argument-declaration-ignored", "binary-gpu-declaration-ignored", "estimate-by-exact-command", "leftover-processes-kept", "long-run-not-noted", "resumable-declaration-ignored",
    "attempt-label-not-passed", "peak-read-only-by-sampling", "shard-outputs-not-carried", "probe-children-left", "scene-removed-at-finish", "scene-kept-after-collect",
    "scene-keep-ignored", "scene-left-by-clean", "leftovers-outside-own-unit",
    "shard-siblings-abandoned", "shard-cancel-ends-whole", "shard-missing-unnamed", "parent-cancel-spares-running", "cancel-outcome-unchecked",
    "failed-start-counted", "child-rows-unverified", "task-locks-kept", "card-fault-assigned-card-quarantined",
    "recorded-cap-floored", "sealed-gap-not-backfilled", "sealed-gap-length-ignored", "blocker-remaining-unbounded",
    "item-estimate-skipped", "item-done-unrecorded", "item-estimate-by-command", "item-rate-missing-skipped", "estimate-failure-holds-parent",
    "gap-fill-room-ignored", "estimate-basis-unreported", "estimate-basis-sentence-fixed", "shard-plan-merge-unestimated", "shard-pieces-unestimated",
    "attention-queue-unreported", "attention-overrun-unreported", "attention-card-replaced-unreported", "attention-finished-unreported", "attention-cleanup-unreported",
    "attention-shard-advance-unreported", "attention-reported-every-wait", "temporary-service-build-unrecognized",
    "pause-queue-ignored", "pause-start-ungated", "pause-halts-reconcile", "pause-state-memory-only", "resume-leaves-cache-paused", "pause-health-unreported",
    "pause-waiting-unreported", "pause-judges-starved",
    "pause-allow-ignored", "pause-allow-matches-any-session", "pause-allow-waiting-paused", "pause-allow-change-ignored", "resume-keeps-allowed-session",
    "pause-allow-memory-only", "pause-allow-unsigned", "pause-allow-unreported", "pause-other-running-miscounted",
    "wait-limit-fixed", "temporary-service-own-server-unrecognized",
    "shard-siblings-unshared", "build-family-ignored", "family-arguments-unfilled", "retry-build-directory-fresh", "shard-key-from-joined-command",
    "workspace-write-interrupted-unreported", "apply-outcome-ignored", "gpu-card-count-ignored",
    "history-peak-ignored", "read-only-arguments-counted-as-writer", "peer-retry-build-directory-fresh",
    "history-peak-unwindowed", "whole-seals-every-host", "card-whole-counts-free-cards-only", "family-cap-hit-not-counted", "parallelism-falls-back-to-cores",
    "at-cap-busy-cpu-kept-running", "at-cap-fraction-ignored", "peer-memory-line-unread", "aged-whole-not-counted", "aged-fragment-bypasses-seal",
    "attention-quiet-busy-unreported", "io-wait-ignored", "attention-io-wait-unreported", "stall-card-load-ignored", "card-presence-counts-as-busy",
    "attention-gpu-busy-quiet-unreported",
    "health-head-judged-apart", "progress-unwatched", "progress-gates-ignored", "card-wait-blocks-later-cards",
    "whole-workspace-cargo-accepted", "workspace-writer-headroom-skipped", "gap-fill-room-counts-all-running", "fragment-group-never-drops",
    "repository-target-bypass-unwarned", "cancelled-artifacts-kept",
    "inbox-not-sent", "inbox-auth-omitted", "inbox-reported-every-round", "pushed-outcome-not-collected",
    "health-unwatched", "health-alerted-every-round", "unclaimed-unwarned",
    "cancel-not-started-receipt-unread", "quiet-log-busy-any-exemption", "inbox-pending-dropped-on-error",
    "leftovers-error-uncaught", "inbox-attention-per-shard",
    "memory-peak-with-page-cache", "attention-reported-memory-only", "attention-queue-reasonless-sent",
    "aged-seal-gap-filled", "budget-over-maxima-waits", "oversized-budget-silent", "long-run-idle-silent",
    "fills-any-cap-silent", "old-metric-peaks-read",
    "attention-next-generic", "unclaimed-outcome-not-repushed", "anomaly-not-repushed", "inbox-socket-from-submit-only", "inbox-missing-accepted",
    "confirm-skipped", "confirm-counts-unchecked", "unconfirmed-kept", "family-lends-run-peak", "cleanup-tests-tables-hand-listed",
    "seal-host-that-cannot-fit", "build-phase-past-finished", "fills-any-cap-counts-cap-hits", "path-check-substring",
    "card-idle-stall-threshold-ignored", "peer-health-single-timeout-reported", "cap-hit-stat-unreported",
    "over-two-hours-not-bounced", "over-two-hours-cap-hits-counted", "over-two-hours-commit-exempt-by-command-only",
    "requeue-keeps-original-position", "long-run-confirmed-still-bounced", "long-run-confirmed-flag-hidden",
    "whole-package-unconfirmed-accepted", "whole-package-confirmed-exempts-two-hours", "whole-package-flag-dropped-on-requeue")


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


def temporary_service_wrapped(command, environment=None):
    """命令包着临时 prune-server 的那一层（unwrapped_temporary_service 剥得开）交 True：命令键前面另接 TEMPORARY_SERVICE_KEY_MARK（task_server.py 的 command_key）。
    弄坏开关 temporary-service-key-shared 下一律 False（包着的与不包的共键，第十二批甲的样子）。"""
    if broken("temporary-service-key-shared", environment):
        return False
    return unwrapped_temporary_service(list(command), environment) != list(command)


def unwrapped_temporary_service(command, environment=None):
    """剥开临时 prune-server 的包装（TEMPORARY_SERVICE_SCRIPT --with-temporary-service [--server-binary <路径>] -- <原命令>，前面的 python3 一起剥；
    包装前面的 env、nice 这类前缀照留）：交前缀加原命令；不是这一层包装的照原样交。算命令键（key_words）、认登记表行
    （task_server.py 的 verdict_library_refusal）、判包装里会编或要卡（task_server.py 的 temporary_service_heavy_refusal）用它；包装本身照旧整条执行。
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
    不另写一份判法；task_server.py 照它把卡的台标签映到 local / peer。"""
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


def peer_shell_timed(host, script, environment=None):
    """经 ssh 在第二台上跑一段 shell（NFS_SSH_OPTIONS）：→ (退出码, 标准输出, 标准错误, 是不是超时)；ssh 起不来或超时退 255。自证里 ssh 换成 TASK_SERVER_PEER_SSH 那一份、
    时限换成 TASK_SERVER_SELFTEST_NFS_SSH_SECONDS。"""
    ssh = selftest_override("TASK_SERVER_PEER_SSH", environment) or "ssh"
    seconds = float(selftest_override("TASK_SERVER_SELFTEST_NFS_SSH_SECONDS", environment) or NFS_SSH_SECONDS)
    try:
        completed = subprocess.run([ssh, *NFS_SSH_OPTIONS, host, script], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=seconds)
    except subprocess.TimeoutExpired as error:
        return 255, "", f"{type(error).__name__}: {error}", True
    except OSError as error:
        return 255, "", f"{type(error).__name__}: {error}", False
    return completed.returncode, completed.stdout, completed.stderr, False


def peer_shell(host, script, environment=None):
    """peer_shell_timed 去掉是不是超时那一样：→ (退出码, 标准输出, 标准错误)。"""
    code, output, error, _ = peer_shell_timed(host, script, environment)
    return code, output, error


def peer_shell_rechecked(host, script, environment=None):
    """连第二台一次超时先当慢（文件头「NFS 共享」的挂载）：→ (退出码, 标准输出, 标准错误, 慢的那一句)。头一次超时的隔 NFS_RECHECK_PAUSE_SECONDS 复查一次，
    复查回了照复查的交、慢的那一句写「慢：头一次 ssh 超时 …，隔 … 秒复查回了」；复查还超时或连不上的交 255、标准错误写两次的原因。头一次没超时的照它交、不复查。
    弄坏开关 peer-health-single-timeout-reported 下不复查（头一次超时就照连不上交）。"""
    code, output, error, timed_out = peer_shell_timed(host, script, environment)
    if not timed_out or broken("peer-health-single-timeout-reported", environment):
        return code, output, error, ""
    seconds = float(selftest_override("TASK_SERVER_SELFTEST_NFS_SSH_SECONDS", environment) or NFS_SSH_SECONDS)
    pause = float(selftest_override("TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS", environment) or NFS_RECHECK_PAUSE_SECONDS)
    time.sleep(pause)
    again, output, second_error, second_timed_out = peer_shell_timed(host, script, environment)
    if again == 255:
        why = "复查又超时" if second_timed_out else f"复查连不上（{last_line(second_error, '没有输出')}）"
        return 255, output, f"ssh 超时 {seconds:g} 秒，隔 {pause:g} 秒{why}", ""
    return again, output, second_error, f"慢：头一次 ssh 超时 {seconds:g} 秒（{last_line(error, '没有输出')}），隔 {pause:g} 秒复查回了"


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


def nfs_probe(root, host, environment=None):
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
        code, seen, error = peer_shell(host, reader, environment)
        if code != 0:
            return f"本机→第二台不过：本机写的 {local_file}，第二台读不到（退 {code}：{last_line(error, '没有输出')}）"
        if seen != token:
            return f"本机→第二台不过：本机写的 {local_file}，第二台读回的不是同一个记号"
        if broken("nfs-probe-one-way", environment):
            return NFS_PASSED
        token = secrets.token_hex(16)
        code, _, error = peer_shell(host, f"printf %s {token} > {shlex.quote(peer_file)}", environment)
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


def nfs_mount_verdict(root, host, environment=None, slow=None):
    """「挂载」那一项（文件头「NFS 共享」）：经 ssh 在第二台上 stat -f 这个根下面的「.」——走进目录会触发 x-systemd.automount 挂上它（自动挂载点闲着时
    只是一层 autofs 的占位，mountpoint -q 照样退 0，不说明 NFS 挂上了），读到的文件系统类型是 nfs 才过。→「过」或一句为什么不过、怎么在第二台恢复。
    ssh 一次超时照 peer_shell_rechecked 复查一次：复查回了照回的判，慢的那一句（带上根）追加进 slow（给了的话）；复查仍不通的写连不上。
    弄坏开关 nfs-mount-placeholder-accepted 下照改前只看 mountpoint -q（占位也算挂上）。"""
    if broken("nfs-mount-placeholder-accepted", environment):
        code, _, error, slowly = peer_shell_rechecked(host, f"mountpoint -q -- {shlex.quote(root)}", environment)
        if slowly and slow is not None:
            slow.append(f"共享根 {root} 的「挂载」{slowly}")
        return NFS_PASSED if code == 0 else (f"第二台 mountpoint -q 退 {code}：没把它挂在同一路径" if code != 255
                                             else f"连不上第二台（{last_line(error, '没有输出')}）")
    code, output, error, slowly = peer_shell_rechecked(host, f"{NFS_MOUNT_PROBE_TEXT} {shlex.quote(root + '/.')}", environment)
    if slowly and slow is not None:
        slow.append(f"共享根 {root} 的「挂载」{slowly}")
    if code == 255:
        return f"连不上第二台（{last_line(error, 'ssh 退 255，没有输出')}）"
    kind = output.strip()
    if code == 0 and kind in NFS_FILESYSTEM_TYPES:
        return NFS_PASSED
    found = f"stat -f 退 {code}：{last_line(error, '没有输出')}" if code != 0 else f"那里的文件系统是 {kind or '（空）'}、不是 NFS"
    return f"第二台读不到挂上的 NFS（{found}）：{NFS_MOUNT_RECOVERY.format(root=root)}"


def nfs_share_report(configuration_path, environment=None, probe=True):
    """NFS 共享判得过没有（文件头「NFS 共享」）：→ {ok, problem, next_step, roots: [{root, directory, export, mount, probe}…], checked_at, slow}。
    slow 是「挂载」那一项 ssh 头一次超时、复查回了的那几句（nfs_mount_verdict；服务只记进服务日志）。
    每一项是「过」或一句为什么不过（没判的写「没判：…」）；problem 是第一处不过的那一句（哪个根、哪一项、为什么）。probe=False 不做两向探针。
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
        entry["directory"] = NFS_PASSED if os.path.isdir(root) else "本机没有这个目录"
        entry["export"] = NFS_PASSED if root in exports else (exports_why or f"{selftest_override('TASK_SERVER_NFS_EXPORTS', environment) or NFS_EXPORTS_FILE} 里没有第一列是它的导出行")
        if not host:
            entry["mount"] = f"{host_why}：连不了第二台"
        else:
            entry["mount"] = nfs_mount_verdict(root, host, environment, report["slow"])
        if not probe:
            entry["probe"] = "没判：只在起来时与 control check、install、start 做"
        elif entry["directory"] != NFS_PASSED or entry["mount"] != NFS_PASSED:
            entry["probe"] = "没判：本机目录或第二台挂载没过"
        else:
            entry["probe"] = nfs_probe(root, host, environment)
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


def command_word_index(argv):
    """剥掉 env（连同 名=值 与 -u 名）、nice -n N、timeout <时长>、setsid、bash / sh / python3 起脚本、capped.sh N 之后命令词在 argv 里的位置；剥完了没有交 len(argv)。"""
    index = 0
    while index < len(argv):
        word = argv[index]
        name = os.path.basename(word)
        if name == "env":
            index += 1
            while index < len(argv) and ("=" in argv[index] or argv[index] in ("-u", "--unset") or argv[index].startswith("-")):
                index += 2 if argv[index] in ("-u", "--unset") else 1
            continue
        if name == "nice":
            index += 3 if index + 1 < len(argv) and argv[index + 1] == "-n" else 1
            continue
        if name == "timeout":
            index += 1
            while index < len(argv) and argv[index].startswith("-"):
                index += 1
            index += 1
            continue
        if name == "setsid":
            index += 1
            while index < len(argv) and argv[index].startswith("-"):
                index += 1
            continue
        if name in ("bash", "sh", "python3", "python"):
            index += 1
            while index < len(argv) and argv[index].startswith("-") and argv[index] != "-c":
                index += 1
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
    """脚本文件头前 DECLARED_HEADER_LINES 行里以 mark 起头的那一行写的词（分片、要卡的参数、续跑三种声明共用）；文件读不了、不是普通文件或没写交 None。"""
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
    """要卡的参数与续跑两种声明的判法：声明了 * 就是任何参数都算，不然第一个非选项参数（first_non_option）要是声明的词之一。"""
    if not declared:
        return False
    return "*" in declared or first_non_option(arguments) in declared


DECLARED_BINARY_GPU_ARGUMENTS_MARK = "// task-server-gpu-arguments:"   # 编出来的二进制：同名 src/bin/<名字>.rs 文件头那一行（文件头「GPU」）
BINARY_SOURCE_ROOTS = ("crates", "scripts/experiment-apparatus")   # 去哪几处找二进制的同名源（各自下面任一层的 src/bin/）


def compiled_binary_source(binary_path, environment=None):
    """命令位置上的 <编译目录>/release/<名字> 或 …/debug/<名字> → 仓里唯一那一份同名源（src/bin/<名字>.rs 或 src/bin/<名字>/main.rs）；
    不是这种路径、找不到或不止一份交 None。仓根在自证里换成 TASK_SERVER_FAKE_REPOSITORY_ROOT（文件头「只在自证里认的变量」）。"""
    if os.path.basename(os.path.dirname(binary_path)) not in ("release", "debug"):
        return None
    name = os.path.basename(binary_path)
    repository = selftest_override("TASK_SERVER_FAKE_REPOSITORY_ROOT", environment) or REPOSITORY_ROOT
    found = []
    for root in BINARY_SOURCE_ROOTS:
        for directory, subdirectories, files in os.walk(os.path.join(repository, root)):
            subdirectories[:] = [entry for entry in subdirectories if entry not in ("target", ".git", "node_modules") and not entry.startswith("target-")]
            if os.path.basename(directory) == "bin" and os.path.basename(os.path.dirname(directory)) == "src":
                if name + ".rs" in files:
                    found.append(os.path.join(directory, name + ".rs"))
                if name in subdirectories and os.path.isfile(os.path.join(directory, name, "main.rs")):
                    found.append(os.path.join(directory, name, "main.rs"))
    return found[0] if len(found) == 1 else None


def declared_gpu_binary(binary_path, arguments, environment=None):
    """编出来的二进制按同名源文件头的「// task-server-gpu-arguments: <词>…」认要不要卡，判法同 declared_gpu_arguments；
    找不到唯一的同名源、没写那一行交 False。弄坏开关 binary-gpu-declaration-ignored 下一律 False（改前认不出裸二进制要卡）。"""
    if broken("binary-gpu-declaration-ignored", environment) or broken("gpu-argument-declaration-ignored", environment):
        return False
    source = compiled_binary_source(binary_path, environment)
    return bool(source) and declaration_matches(arguments, declared_header_words(source, DECLARED_BINARY_GPU_ARGUMENTS_MARK))


def declared_gpu_arguments(script_path, arguments, environment=None):
    """命令位置上起的脚本按文件头的声明认要不要卡（文件头「GPU」）：「# task-server-gpu-arguments: <词>…」里有这一条的第一个非选项参数（或写了 *）交 True；
    文件不在、没有那一行交 False。弄坏开关 gpu-argument-declaration-ignored 下一律 False（改前只按名字认）。"""
    if broken("gpu-argument-declaration-ignored", environment):
        return False
    return declaration_matches(arguments, declared_header_words(script_path, DECLARED_GPU_ARGUMENTS_MARK))


def declared_gpu_cards(sources):
    """要几张卡（文件头「GPU」的张数）：sources 是要读的源文件（脚本读「# task-server-gpu-cards:」，.rs 读「// task-server-gpu-cards:」），取声明的最大值；
    都没写是一张。→ (张数, 写坏的那一句)：写的不是一个正整数的不算、点名哪一份写了什么（服务照 run-condition-unmet 打回）。"""
    wanted, problems = 1, []
    for path in sources:
        declared = declared_header_words(path, DECLARED_SOURCE_GPU_CARDS_MARK if path.endswith(".rs") else DECLARED_GPU_CARDS_MARK)
        if declared is None:
            continue
        if len(declared) != 1 or not declared[0].isdigit() or int(declared[0]) < 1:
            problems.append(f"{path} 文件头的张数声明写成「{' '.join(declared)}」，要写一个正整数")
            continue
        wanted = max(wanted, int(declared[0]))
    return wanted, "；".join(problems)


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


def cargo_build_family(arguments, workspace=""):
    """cargo 这一层的编译族（文件头「资源记录」的编译族）：包（去重排序）、档（--release 与 --profile release 同一档，没写是 dev）、特性（去重排序）、
    --all-features、--no-default-features、--target；子命令、测试目标名、测试名筛选、`--` 之后的 libtest 参数不进。一个包都没点名的记 cargo 起的那个目录
    （workspace：相对它所在工作树的根，调用方交进来）。→ 「〈编译族〉 …」一段文本。"""
    values = cargo_option_values(arguments)
    parts = [f"-p {name}" for name in sorted(set(values["-p"]))] or [f"目录 {workspace or '.'}"]
    parts.append(f"--profile {values['--profile']}")
    if values["--features"]:
        parts.append("--features " + ",".join(sorted(set(values["--features"]))))
    parts += [flag for flag in ("--all-features", "--no-default-features") if values.get(flag)]
    if values["--target"]:
        parts.append(f"--target {values['--target']}")
    return f"{BUILD_FAMILY_MARK} " + " ".join(parts)


def option_value_in_arguments(arguments, option):
    """参数里 option 的值：认「option 值」与「option=值」，`--` 之后的不认，取第一处；没有交 None。"""
    for index, word in enumerate(arguments):
        if word == "--":
            return None
        if word == option:
            return arguments[index + 1] if index + 1 < len(arguments) else None
        if word.startswith(option + "="):
            return word[len(option) + 1:]
    return None


def filled_family_words(declared, arguments, environment=None):
    """编译族声明里取自命令参数的词照这一条命令填上（文件头「资源记录」的编译族）：「$<n>」换成 arguments 里第 n 个词（从 1 数），「$<以 - 起头的选项名>」换成
    那个选项的值（option_value_in_arguments），别的词照原样。→ 填好的词表；有一个填不上（没有那个词、值是空的或以 - 起头、$ 后面既不是正整数也不是选项名）交 None。
    弄坏开关 family-arguments-unfilled 下照原样交回、一个都不填（改前的样子：「-p $3」照字面算，每个包都归进同一个字面的族）。"""
    if broken("family-arguments-unfilled", environment):
        return list(declared)
    filled = []
    for word in declared:
        if not word.startswith(FAMILY_ARGUMENT_MARK) or word == FAMILY_ARGUMENT_MARK:
            filled.append(word)
            continue
        reference = word[len(FAMILY_ARGUMENT_MARK):]
        if reference.isdigit() and int(reference) >= 1:
            value = arguments[int(reference) - 1] if int(reference) <= len(arguments) else None
        elif reference.startswith("-"):
            value = option_value_in_arguments(arguments, reference)
        else:
            return None
        if not value or value.startswith("-"):
            return None
        filled.append(value)
    return filled


def declared_build_family(argv, cwd, environment=None):
    """命令位置上起的入口脚本文件头写了「# task-server-cargo-family: <cargo 参数>…」的，照声明的参数算编译族（cargo_build_family）；
    声明里取自命令参数的词（$<n>、$<选项名>）先照这一条命令填上（filled_family_words），填不上的交空串（不归族）。
    单独 --selftest、带 --list-items / --help / -h 的、没写那一行的交空串。"""
    index = command_word_index(argv)
    if index >= len(argv):
        return ""
    arguments = argv[index + 1:]
    if (arguments[:1] and arguments[0] in NOT_SHARDED_ARGUMENTS) or "--selftest" in arguments:
        return ""
    declared = declared_header_words(os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, argv[index])), DECLARED_BUILD_FAMILY_MARK)
    if not declared:
        return ""
    filled = filled_family_words(declared, arguments, environment)
    return cargo_build_family(filled) if filled else ""


def package_directories(names, environment=None):
    """仓里（自证里换成 TASK_SERVER_FAKE_REPOSITORY_ROOT）BINARY_SOURCE_ROOTS 下 Cargo.toml 的 [package] name 是 names 里之一的那几个目录：→ {包名: 目录}。"""
    wanted = set(names)
    repository = selftest_override("TASK_SERVER_FAKE_REPOSITORY_ROOT", environment) or REPOSITORY_ROOT
    found = {}
    for root in BINARY_SOURCE_ROOTS:
        for directory, subdirectories, files in os.walk(os.path.join(repository, root)):
            subdirectories[:] = [entry for entry in subdirectories if entry not in PACKAGE_SKIPPED_DIRECTORIES and not entry.startswith("target-")]
            if "Cargo.toml" not in files:
                continue
            try:
                with open(os.path.join(directory, "Cargo.toml"), encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
            except OSError:
                continue
            section = re.search(r"^\[package\]\s*$(.*?)(?=^\[|\Z)", text, re.MULTILINE | re.DOTALL)
            named = re.search(r'^name\s*=\s*"([^"]+)"', section.group(1), re.MULTILINE) if section else None
            if named and named.group(1) in wanted:
                found.setdefault(named.group(1), directory)
    return found


def cargo_card_sources(arguments, environment=None):
    """cargo test / run / bench 点名的包里要读张数声明的源（文件头「GPU」的张数）：点了 --test <名> 的读 tests/<名>.rs（或 tests/<名>/main.rs），
    只点 --lib 的读 src/lib.rs，别的读 src/lib.rs、src/main.rs、src/bin/ 下的与 tests/ 下的。没点名包（-p）的不读。→ [路径…]（只列在的）。"""
    values = cargo_option_values(arguments)
    sources = []
    for directory in package_directories(values["-p"], environment).values():
        if values["--test"]:
            candidates = [os.path.join(directory, "tests", name + ".rs") for name in values["--test"]]
            candidates += [os.path.join(directory, "tests", name, "main.rs") for name in values["--test"]]
        elif values.get("--lib") and not values.get("--tests"):
            candidates = [os.path.join(directory, "src", "lib.rs")]
        else:
            candidates = [os.path.join(directory, "src", "lib.rs"), os.path.join(directory, "src", "main.rs")]
            for subdirectory in (os.path.join(directory, "src", "bin"), os.path.join(directory, "tests")):
                with contextlib.suppress(OSError):
                    for name in sorted(os.listdir(subdirectory)):
                        candidates.append(os.path.join(subdirectory, name, "main.rs") if os.path.isdir(os.path.join(subdirectory, name))
                                          else os.path.join(subdirectory, name))
        sources += [path for path in candidates if path.endswith(".rs") and os.path.isfile(path)]
    return sources


def resumable_command(argv, cwd, environment=None):
    """这条命令登记了续跑没有（文件头「续跑」）：命令位置上起的脚本（剥掉 env、nice、timeout、setsid、bash / sh / python3、capped.sh N 之后，同 sharded_entry）
    文件头写了「# task-server-resumable: <词>…」、第一个非选项参数是其中一个（或写了 *）。弄坏开关 resumable-declaration-ignored 下一律当没登记。"""
    if broken("resumable-declaration-ignored", environment):
        return False
    index = command_word_index(argv)
    if index >= len(argv):
        return False
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, argv[index]))
    return declaration_matches(argv[index + 1:], declared_header_words(script, DECLARED_RESUMABLE_MARK))


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


def item_entry(argv, cwd):
    """按项估预计用时的入口（文件头「资源记录」的预计用时）：命令起的是 ITEM_ENTRIES 里的一条（剥法同 sharded_entry），交 (仓根下的相对路径, [点到的项…])；
    不是这三条、或带了 NOT_SHARDED_ARGUMENTS 里的参数（--selftest、--list-items、--help、-h）的交 ("", [])。「--item <项>」与「--item=<项>」都认。"""
    index = command_word_index(argv)
    if index >= len(argv):
        return "", []
    word, arguments = argv[index], argv[index + 1:]
    script = os.path.realpath(os.path.join(cwd, word))
    entry = next((relative for relative in ITEM_ENTRIES if script.endswith("/" + relative) or word == relative), "")
    if not entry or any(argument in NOT_SHARDED_ARGUMENTS for argument in arguments):
        return "", []
    items = []
    for position, argument in enumerate(arguments):
        if argument == ITEM_ARGUMENT and position + 1 < len(arguments):
            items.append(arguments[position + 1])
        elif argument.startswith(ITEM_ARGUMENT + "="):
            items.append(argument[len(ITEM_ARGUMENT) + 1:])
    return entry, items


def estimate_command(argv):
    """估算子任务起的那一条：同一条入口，紧跟在脚本后面接 ITEM_ESTIMATE_ARGUMENTS，原命令的参数（几个 --item 与后面的根）照原样跟在后面。"""
    index = command_word_index(argv)
    return list(argv[:index + 1]) + list(ITEM_ESTIMATE_ARGUMENTS) + list(argv[index + 1:])


def item_rate_key(entry, item):
    """item_rates 表的键（command_key 那一列）：入口脚本（仓根下的相对路径）与项名，用制表符连。"""
    return f"{entry}\t{item}"


def item_lines(path):
    """一次尝试的整份日志里每项那几行（文件头「资源记录」的预计用时）：→ ({项: {"to_judge", "reusable"}}（同一项几行取最后一行），
    [每行 ITEM_DONE 一份 {"item", "seconds", "judged", "reused", "exit"}])。只看去掉首尾空白之后以 ITEM_ 起头的行；按行流着读，不整份读进内存；读不了交 ({}, [])。"""
    plans, dones = {}, []
    try:
        with open(path, "rb") as handle:
            for raw in handle:
                stripped = raw.strip()
                if not stripped.startswith(ITEM_LINE_PREFIX):
                    continue
                text = stripped.decode("utf-8", "replace")
                found = ITEM_PLAN_LINE.match(text)
                if found:
                    plans[found.group(1)] = {"to_judge": int(found.group(2)), "reusable": int(found.group(3))}
                    continue
                found = ITEM_DONE_LINE.match(text)
                if found:
                    dones.append({"item": found.group(1), "seconds": float(found.group(2)), "judged": int(found.group(3)), "reused": int(found.group(4)),
                                  "exit": int(found.group(5))})
    except OSError:
        return {}, []
    return plans, dones


def shard_child_key(base_key_text, role, index, count, environment=None):
    """子任务的命令键原文（交给内存包装的 RUN_WITH_MEMORY_CAP_KEY）：这件的键加「〈task-server 分片 <步> 第<片号>片共<片数>片〉」。
    加的那一段不带 /：键的换算（memory_peaks_lib.py 文件头「键」③）把带 / 的位置参数折成〈文件〉，带了各片就落一个键。
    弄坏开关 shard-key-position-folded 下照改前写成 <片号>/<片数>。"""
    if role != "shard":
        position = ""
    elif broken("shard-key-position-folded", environment):
        position = f" {index}/{count}"
    else:
        position = f" 第{index}片共{count}片"
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
    if exit_code == 78 and CARD_FAULT_MARK in (log_text or "") and not broken("card-fault-unread", environment):
        return "card-fault"
    if exit_code == 255 and host == "peer" and not broken("peer-exit-unsplit", environment):
        return "peer-run-failed"
    if exit_code == 75 and broken("prune-server-exit-unsplit", environment):
        return "command-failed"   # 弄坏开关：改前的样子，75 不单列
    if exit_code in EXIT_TYPES:
        return EXIT_TYPES[exit_code]
    if broken("unlisted-exit-swallowed", environment):
        return "succeeded"
    return "command-failed"


def card_fault_cards(log_text):
    """清场判出的坏卡清单（文件头「错误类型」card-fault 那一行）：日志里最后一行以 CARD_FAULT_MARK 起头的清场行的 faulty-cards= 一栏 →
    ["<台>:<序号>", …]（序号是整数的那几项，按出现次序、去重）；没有这一行、没有这一栏、一项都认不出交 []。"""
    lines = [line.strip() for line in (log_text or "").splitlines() if line.strip().startswith(CARD_FAULT_MARK)]
    if not lines:
        return []
    found = []
    for word in lines[-1].split():
        if not word.startswith(CARD_FAULT_LIST_FIELD):
            continue
        for item in word[len(CARD_FAULT_LIST_FIELD):].split(","):
            address, _, index = item.rpartition(":")
            entry = f"{address}:{int(index)}" if address and index.isdigit() else ""
            if entry and entry not in found:
                found.append(entry)
    return found


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


def reason_fields(task, attempts, log_text):
    """打回的理由（文件头「错误类型」表后的「打回的理由」）：→ [(栏, 值)]，type 在头、why 与 next 在尾，中间是 REASON_ELEMENTS 给这一种的要素。
    attempts 是结局所在的那几次尝试（分片的是没成那一步的子任务的），log_text 是最后那一次日志的尾巴。"""
    outcome = task.get("outcome") or ""
    task_id = task.get("id")
    note = (task.get("note") or "").strip()
    last = attempts[-1] if attempts else {}
    lines = log_lines(log_text)
    quiesce = next((line for line in reversed(lines) if line.startswith(GPU_QUIESCE_LINE_MARK)), "")
    faulty = "、".join(last.get("faulty_cards") or [])   # 清场点名的坏卡（文件头「错误类型」card-fault 那一行），不是分到的那张
    facts = {
        "exit": str(task.get("exit_code")),
        "host": HOST_NAMES.get(last.get("host") or task.get("maximum_host") or "", last.get("host") or "本机"),
        "last_line": log_last_line(log_text),
        "cap": last.get("memory_cap_text") or "没起",
        "peak": size_text(last.get("memory_peak")) if attempts else (task.get("maximum_peak_text") or "没量到"),   # 一次都没起的照打回时记的预测峰值
        "reached": task.get("maximum_cap_text") or last.get("memory_cap_text") or "?",
        "by": last.get("squeezed_by") or "同台没有别的任务在跑（slice 里回收不掉的用量把它挤掉）",
        "card": last.get("card") or "没分到卡",
        "quiesce": quiesce[:300] if quiesce else ("没清场：一次都没起" if not attempts else "日志里没有清场那一行"),
        "unit": last.get("unit") or "没起单元",
        "judged": note or "没有说明",
        "rule": note or "没有说明",
        "what": note or "任务记录",
    }
    if outcome == "cancelled":
        facts["by"] = "提交它的会话（task-server/task cancel）"
    counts = task.get("outcome_counts") or {}
    whys = {
        "succeeded": "命令退 0",
        "command-failed": f"命令自己退 {facts['exit']}（不是服务判的；日志最后一行见 last_line）",
        "memory-cap-hit": f"撞了这一次的内存上限 {facts['cap']}（量到的峰值 {facts['peak']}），撞了 {counts.get('memory-cap-hit', 1)} 次" + (f"：{note}" if note else ""),
        # 一次都没起的（记账超过每一台可用上限，task_server.py 文件头 ②「预算」）：说明就是 note，峰值写 note 里预测的那一个
        "memory-cap-at-maximum": ((f"{facts['host']}给到 {facts['reached']} 还撞（峰值 {facts['peak']}），再乘 1.5 就到能派的每台的总上限" + (f"：{note}" if note else ""))
                                  if attempts else (note or "记账超过能派的每一台的可用上限，一次都没起")),
        "squeezed-out": f"{facts['host']}的 slice 总上限满了，这一条（上限 {facts['cap']}）被内核挤掉 {counts.get('squeezed-out', 1)} 次；同台在跑的：{facts['by']}",
        "gpu-memory-short": f"卡 {facts['card']} 显存不够（内存包装退 245），重排过也没够",
        "timed-out": f"内存包装的限时到了（253，上限 {facts['cap']}）",
        "wrapper-refused": "内存包装用法错或 scope 起不来（251）",
        "run-condition-unmet": note or f"命令或清场包装退 78（运行条件不满足）：{facts['last_line']}",
        "not-run": "命令报本次未跑（77）",
        "peer-run-failed": "第二台那一趟没跑成（multi-host-run.sh 退 255、日志里没有第二台那条命令的结束行：连不上、ssh 被重置或那边的准备没过），重排过也没成",
        "prune-server-unavailable": "命令自己退 75：prune-server（判定库服务）不可用；服务另起了一次 python3 prune-server/control check，不通的写了告警",
        "card-fault": ((f"清场判出卡 {faulty} 坏了（这一次分到的是 {facts['card']}）" if faulty else
                        f"清场判出有卡坏，清场那一行里取不到坏卡清单（faulty-cards=），一张卡都没停用（这一次分到的是 {facts['card']}）") + (f"：{note}" if note else "")),
        "launcher-lost": f"尝试的单元 {facts['unit']} 不在了、又没留结果，服务对不回来",
        "cancelled": note or "提交它的会话取消、单元已停",
        "record-tampered": f"库里这条记录校验不过（{facts['what']}），一次都不起",
        "never-fits": f"哪台都等多久也放不下，或点名第二台而它用不了：{facts['judged']}",
        "unit-start-failed": f"尝试的单元起不来：{facts['judged']}",
        "resubmit-required": f"服务规则改了，还在排的这一条照现行提交规则不再收：{facts['rule']}",
        "stalled": f"卡住，服务停了它：{facts['judged']}",
        "shard-plan-refused": f"分片的计划那一步退 0 而它写的计划读不通：{facts['judged']}",
        "over-two-hours": f"不是提交时的门禁，累计跑满两小时，服务停了它：{facts['judged']}",
    }
    why = whys.get(outcome, note or f"结局 {outcome}")
    # 出路照异常表这一种那一行的修改意见（文件头「异常表」）；memory-cap-at-maximum 前面另接撞了哪台、到了多少（maximum_next_step）
    if outcome == "succeeded":
        next_step = "无"
    elif outcome == "memory-cap-at-maximum":
        next_step = maximum_next_step(task.get("maximum_host") or last.get("host") or "", facts["reached"], facts["peak"])
    elif outcome == "launcher-lost":
        next_step = anomaly_advice(outcome, task_id).replace("<unit= 点名的单元>", facts["unit"])
    elif outcome == "over-two-hours" and task.get("requeue_command"):   # 对的那一条出路照这一回服务算的两个数写成整行（task_server.py 的 requeue_command_text）
        next_step = f"{anomaly_advice(outcome, task_id)}；对的照这一行：{task['requeue_command']}"
    else:
        next_step = anomaly_advice(outcome, task_id)
    return [("type", outcome)] + [(name, facts[name]) for name in REASON_ELEMENTS.get(outcome, ())] + [("why", why), ("next", next_step)]


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
        peak_text = (size_text(log_attempts[-1].get("memory_peak")) if log_attempts
                     else failed.get("maximum_peak_text") or task.get("maximum_peak_text") or "没量到")
        line += f" host={HOST_NAMES.get(host, host or '本机')} reached={cap_text or '?'} next={maximum_next_step(host, cap_text or '?', peak_text)}"
    return line


def suspect_line(task_id, suspect):
    """报疑似卡住的那一行 SUSPECT_MARK（task 文件头 wait 那一段）。"""
    return (f"{SUSPECT_MARK} task={task_id} attempt={suspect.get('attempt')} host={HOST_NAMES.get(suspect.get('host'), suspect.get('host') or '本机')} "
            f"window={suspect.get('window_seconds')} cores={suspect.get('cores')} log={suspect.get('log') or '无'} why={suspect.get('why')}")


def attention_line(task_id, item):
    """报要处置的提醒的那一行 ATTENTION_MARK（task 文件头 wait 那一段）：值照 shlex 引（同 REASON_MARK，shlex.split 拆得回）；part= 只在提醒说的是分片的子任务时写它的任务号。"""
    part = f" part={item.get('task')}" if item.get("task") not in (None, task_id) else ""
    values = [(name, item.get(name)) for name in ("basis", "why", "next")]
    return (f"{ATTENTION_MARK} task={task_id}{part} type={ATTENTION_TYPE} kind={item.get('kind')} tier={item.get('tier')} "
            + " ".join(f"{name}={shlex.quote(str(value))}" for name, value in values))


@dataclass(frozen=True)
class AnomalyRow:
    kind: str
    action: str
    criterion: str
    advice: str
    unhandled: str


def anomaly_rows(text):
    """文件头「异常表」那一段（text 里从 ANOMALY_TABLE_HEAD 起头的那一行起、到下一个空行为止）逐行读成 [AnomalyRow]，次序照文件头；
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
    """{种类: AnomalyRow}：这一份文件头的异常表（anomaly_rows 照 __doc__ 读，读过一次就留着）。"""
    if not ANOMALY_TABLE_LOADED:
        ANOMALY_TABLE_LOADED.append({row.kind: row for row in anomaly_rows(__doc__)})
    return ANOMALY_TABLE_LOADED[0]


def anomaly_advice(kind, task, environment=None):
    """异常表里这一种的修改意见（next= 那一句的头一段），{task} 换成任务号；表里没有这一种抛 KeyError（种类与表对不上是代码错，不悄悄退回一句通用的）。
    弄坏开关 attention-next-generic 下每一种都交 ANOMALY_GENERIC_ADVICE。"""
    template = ANOMALY_GENERIC_ADVICE if broken("attention-next-generic", environment) else anomaly_table()[kind].advice
    return template.replace(ANOMALY_TASK_PLACEHOLDER, str(task))


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


def confirm_needed_line(task_id, items, tasks, timeout_seconds):
    """确认单末尾那一行 CONFIRM_NEEDED_MARK（机器读的；数是服务此刻算的）。"""
    return f"{CONFIRM_NEEDED_MARK} task={task_id} items={items} tasks={tasks} timeout={float(timeout_seconds):.0f}"


def confirm_needed_fields(text):
    """text 里最后一行 CONFIRM_NEEDED_MARK 的 (任务号, 项数, 条数)；没有交 None。"""
    found = list(CONFIRM_NEEDED_LINE.finditer(text or ""))
    return tuple(int(value) for value in found[-1].groups()) if found else None


def named_scope(command, environment=None):
    """确认单的项数（task_server.py 文件头 ⑦）：命令（key_words 剥过不改跑法的前缀与临时 prune-server 包装）里点名的 --item、--check（值照逗号拆）、
    --test、--bin、--lib 与 `--` 之后带了 --exact 时的测试名（LIBTEST_VALUE_OPTIONS 的值不算）各几个 → [(选项, 值)…]，次序照命令；一个都没点的交 []（整包、不指定范围）。"""
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


def named_scope_text(named):
    """named_scope 交的那几项写成「--item 3、--test 1」（各选项照头一次出现的次序）；一个都没有写「整包、不指定范围」。"""
    counts = {}
    for name, _ in named:
        counts[name] = counts.get(name, 0) + 1
    return "、".join(f"{name} {count}" for name, count in counts.items()) or "整包、不指定范围"


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


def oom_stat_note(log_text):
    """日志里最后一行 OOM_STAT_LINE 换成给人看的一句（撞上限那一刻 scope 的内存明细；读不到的那一项写读不到）；没有交空串。
    大头在 file（页缓存，含没写回的脏页）的另写一句「大头是页缓存」，在 anon 的写「大头是进程自己的内存（堆）」：只记、不改判法。"""
    found = None
    for line in log_lines(log_text):
        matched = OOM_STAT_LINE.match(line)
        if matched:
            found = matched
    if found is None:
        return ""
    anon, file_pages, dirty, writeback, shared, killed = found.groups()

    def shown(value):
        return "读不到" if value == "unread" else mebibytes_text(int(value))
    text = (f"撞上限那一刻 scope 的内存明细：anon {shown(anon)}、file {shown(file_pages)}（其中没写回的脏页 file_dirty {shown(dirty)}、"
            f"正在写回 file_writeback {shown(writeback)}）、shmem {shown(shared)}，memory.events 的 oom_kill {killed}")
    if anon.isdigit() and file_pages.isdigit():
        text += "：大头是页缓存（file）" if int(file_pages) > int(anon) else "：大头是进程自己的内存（anon，堆）"
    return text


def whole_package_cargo_test(rest):
    """一条 cargo test 后面的参数（rest）是不是整包测试（task_server.py 文件头 ②「提交时拒」⑥）：→ 一句是哪一种，或空串（点名了测试目标）。
    带 --workspace / --all 的是整包；点名了 --test、--bin、或带测试名（`--` 前后不以 - 起头的词、选项的值不算）的不是；--lib 与 --tests 都带、没带测试名的是整包；别的都是。"""
    before, after = (rest[:rest.index("--")], rest[rest.index("--") + 1:]) if "--" in rest else (list(rest), [])
    options, names, index = set(), [], 0
    while index < len(before):
        word = before[index]
        name = word.split("=", 1)[0] if word.startswith("--") else word
        if word.startswith("-"):
            options.add(name)
            if name in CARGO_VALUE_OPTIONS and "=" not in word and not (name in ("-p", "-F", "-j", "-Z") and len(word) > 2):
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


def whole_package_script(argv, cwd):
    """命令位置上起的入口是不是整包测试（task_server.py 文件头 ②「提交时拒」⑥）：→ 一句或空串。
    crash-point-replay.sh 门禁的 --check 把 WHOLE_PACKAGE_CHECK_CELLS 三格全点了的、不给补丁目录的 merge-and-verify.sh 是；带 --selftest、--list-items 这类的不是。"""
    index = command_word_index(argv)
    if index >= len(argv):
        return ""
    word, arguments = argv[index], argv[index + 1:]
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, word))
    if any(argument in NOT_SHARDED_ARGUMENTS for argument in arguments):
        return ""
    if script.endswith("/" + WHOLE_PACKAGE_CRASH_GATE) or word == WHOLE_PACKAGE_CRASH_GATE:
        cells = set()
        for position, argument in enumerate(arguments):
            if argument == "--check" and position + 1 < len(arguments):
                cells.update(arguments[position + 1].split(","))
            elif argument.startswith("--check="):
                cells.update(argument[len("--check="):].split(","))
        if set(WHOLE_PACKAGE_CHECK_CELLS) <= cells:
            return f"{WHOLE_PACKAGE_CRASH_GATE} --check 把 {'、'.join(WHOLE_PACKAGE_CHECK_CELLS)} 三格全点了（验证包整包的测试）"
        return ""
    if script.endswith("/" + WHOLE_PACKAGE_MERGE_ENTRY) or word == WHOLE_PACKAGE_MERGE_ENTRY:
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


def commit_gate_exempt(argv, cwd, task_environment, environment=None):
    """跑满两小时打回的豁免（文件头「错误类型」over-two-hours）：起的是 COMMIT_GATE_ENTRIES 之一（剥法同 sharded_entry）、参数里没有 --check 与 --item
    （「--check=…」这种写法同样算）、范围声明的「为什么」那一段写了 COMMIT_SCOPE_WORD。三样都要。
    弄坏开关 over-two-hours-commit-exempt-by-command-only 下只看起的是不是那几道（带 --check、「为什么」没写提交前的也豁免）。"""
    index = command_word_index(argv)
    if index >= len(argv):
        return False
    word, arguments = argv[index], argv[index + 1:]
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, word))
    if not any(script.endswith("/" + relative) or word == relative for relative in COMMIT_GATE_ENTRIES):
        return False
    if broken("over-two-hours-commit-exempt-by-command-only", environment):
        return True
    if any(argument in SCOPE_RANGE_ARGUMENTS or argument.startswith(tuple(name + "=" for name in SCOPE_RANGE_ARGUMENTS)) for argument in arguments):
        return False
    why = next((segment.split("=", 1)[1] for segment in scope_declaration_segments(task_environment, argv) if segment.startswith("为什么=")), "")
    return COMMIT_SCOPE_WORD in why


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
    """outcome_counts 是含这一次在内每种类型出过几次；maxima 是 {台: 那台的总上限字节或 None（查不到）}。交 RetryDecision（文件头「重试」）。"""
    used = outcome_counts.get(outcome, 0)
    if outcome not in RETRY_LIMITS:
        return RetryDecision(False, outcome=outcome)
    if outcome == "squeezed-out" and broken("squeeze-retry-skipped", environment):
        return RetryDecision(False, outcome=outcome)
    if outcome == "gpu-memory-short" and broken("gpu-short-retry-skipped", environment):
        return RetryDecision(False, outcome=outcome)
    if outcome == "peer-run-failed" and broken("peer-run-retry-skipped", environment):
        return RetryDecision(False, outcome=outcome)
    if outcome == "card-fault" and broken("card-fault-not-replaced", environment):
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
