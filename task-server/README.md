# task-server

**中文** · [English](README.en.md) · [日本語](README.ja.md)

异常退出后的资源占用、远端清理回执与诊断日志见[任务收尾与 GPU 阶段日志](cleanup-protocol.md)。

singlefs 的统一任务服务：一切跑项目代码的任务（轻、重、CPU、GPU）都经它起，进同一个队，由服务按资源与公平规则调度、不另设延迟：排头放不下时后面放得下的可以先起（插队），提交满两小时的排到最前，规则见「## 调度」。任务只由主 agent 交（客户端 `task-server/task`，人手同样用它）；子 agent 只用 `status`、`log` 看结局；挑机器、定内存额度、要不要 GPU、分哪张卡都由服务定，提交时不收这几样。

服务是唯一读写库的进程：FastAPI 跑在 uvicorn 上，作为用户级 systemd 服务 `singlefs-task-server.service`，只开 unix socket（`${XDG_RUNTIME_DIR}/singlefs-task-server/server.sock`，目录 700；全路径至多 107 字节，超了服务、客户端、`control` 都退 78，点名那个路径、它的字节数与上限），不开 TCP。库是内嵌的 SQLite（WAL），放在 `~/.local/state/singlefs/task-server/`，每一行带 HMAC 校验列。内存准入只在服务里判一处：放得下才起，放不下的留在库里排着；起任务仍套 `scripts/hosts/run-with-memory-cap.sh`（本机，设了「已由 task-server 准入」的开关，只做每条上限与 slice 总上限的内核层兜底）或 `scripts/hosts/multi-host-run.sh --peer`（第二台）。

| 文件 | 做什么 |
|---|---|
| `task` | 客户端：`submit [--cwd <目录>] -- <命令…>`、`wait <号>`、`status [<号>] [--json]`、`cancel <号>`、`log <号>`、`clean <号>`；用法与退出码在文件头 |
| `control` | 装与查：`install [--dry-run]`、`check`（服务的 `/health` 只报三样：库写不进；派活判排头能起却连着三轮没起成，判不能起的——等卡、等负载、交替封组这类——不报；看进展：某台照公式放得下排着的最小那条、却 10 分钟一条都没起，兜派活自己判错）、`start`、`stop`、`clear-queue`（服务停着时清掉库里的任务与尝试两张表，先备份库，资源记录不动）、`pin-cap <命令…>` / `unpin-cap <命令…>`（经服务给一条命令钉上或撤掉内存上限的钉值，回读过才退 0）、`pause [--reason <一句>] [--allow-session <会话号>]` / `resume`（暂停与放开排队：暂停着照收提交、不起新任务，只有放行的会话交的照起，见「## 暂停与放开」） |
| `task_server.py` | 服务（`serve`）：接口、排队、准入、起单元、对账、重试、分片、资源记录；判法在文件头 |
| `attempt_runner.py` | 每一次尝试的单元里起命令、记退出码与资源（内存峰值取按轮采到的与内存包装退出时读的里大的）；编译目录任务有了结局由服务删 |
| `task_server_lib.py` | 共用的那一份：路径、依赖、错误类型与退出码表、重试、资源记录、编译目录、分片、环境变量名单、身份；整套的自证 `--selftest` 与弄坏开关表在它文件头 |
| `task_server_selftest.py` | 自证的各格（由上一行起） |
| `singlefs-task-server.service.in` | 用户级单元模板 |

没有自己的配置文件：调度资源照仓根的多机配置 `multi-host.env`（模板 `multi-host.env.example`）读；socket、库、日志、签名密钥、编译目录用 `task_server_lib.py` 文件头「路径」写的固定缺省值。必须依赖：fastapi、uvicorn、`multi-host.env`（含 `NFS_SHARED_ROOTS`、`SHARED_DATA_ROOT` 与 `HOSTS`，写法见「## NFS 共享」的「多机配置写哪几键」）、NFS 共享配好（见「## NFS 共享」）、服务通。

## 调度

挑机器按余量均衡：不要卡的任务在派得过去的几台里挑放得下的，放得下的不止一台时挑放下之后剩余占那台总上限比例大的，一样大取本机；撞上限重排不钉台。分片的各片照计划先去第二台，放得下就不按均衡改台；计划、各片与合并都不钉台，计划里写了 `pin` 的整件记 `shard-plan-refused`（片不钉台，哪台由服务挑）。第二台在同一棵共享的树里跑：仓、工作根 `~/.local/share/singlefs/work/` 所在的共享数据根与会话记录目录经 NFS 两台挂在同一绝对路径（多机配置 `NFS_SHARED_ROOTS`，下面「NFS 共享」一节），派第二台的那一次经 `scripts/hosts/multi-host-run.sh --peer` 在任务的工作目录、同一个绝对路径里直接起，不拷树、不送、不取回、不改写路径，写出的文件两台都读得到；编译目录由 `multi-host-run.sh` 在第二台本地盘上这一趟新建（多机配置 `BUILD_ROOT_BY_HOST` 里第二台那一项下的 `builds/<标签>/target`，不在任何共享根下）、命令结束或被停都先删掉（会编的命令撞上限重排时留着给下一次，见下面「峰值起步」），服务不设、只记下它在哪（`task status` 看得见），`task clean` 不管它。派不派第二台只看第二台此刻用不用得了与余量；派不过去的三种在提交时就拒（下面「## 提交写法」）。没有只派一台的名单（用户：「【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：点到哪个目录、哪个入口的命令都派得到两台，哪台由余量定；派到第二台跑红的照红，由脚本那一侧修，服务不替它挡。改主工作区的工具（`hook-swap.py`、`apply-writer-patch.py`、`stage-mine.py`、`scripts/kb/` 下的、不带 `--skip-apply` 的 `merge-and-verify`）与带 `--write` 的派得到第二台（改的就是同一棵树），撞上限这类本会重排的结局照旧不重排、打回提交它的会话；所以括号里那几样工具首趟就给足：上限不低于「（同键近 5 次最高 + `hook-swap.py` 的 `--verify` 给的验证命令同键近 5 次最高）× 1.5」（不设 4G 底；任务 8367 照同键窗口只给 857M，撞了、换上做了一半）。分片的计划目录与各片的产物目录在共享工作根下的 `task-server-shards/`，两台照同一个绝对路径读写。服务不认合入锁：主工作树正在合入时照常排、照常起。一轮至多在第二台起 4 条（一齐起太多 ssh 会被重置），起满了 2 秒后再排一轮；第二台的结局照 `multi-host-run.sh` 打进日志的那条命令的结束行（`MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>`）定：有这一行按命令自己的退出码定，命令自己退 255 记 `command-failed`、不重排；没有这一行而退 255 才是第二台没跑成（连不上、ssh 被重置），记 `peer-run-failed`、原样重排，至多 2 次。排头放不下时后面放得下的先起（插队）；没满两小时的照整片与碎片按台交替：每台可用上限（总上限减那台此刻的余量）的 85% 为界，记账到界的是整片，别的是碎片；碎片按队列先后装成一组、装到再放一条就超过界时，有整片在等就封组、不再放碎片，整片放得下就起（不等那台腾空），整片起了回到下一组碎片，交替挡住的排队原因带「在等这台腾空」。一条整片只在一台上等腾空、只封那一台：上一轮已为它封了组的接着等那台，不然挑此刻离放得下最近的那台（它的记账减那台照「配置的总上限 − 余量 − 在跑的上限之和」还剩的最小）；别的台照常起碎片。只在它放得下的台上封组：预测峰值本身比一台此刻能给的最大上限还大的，那台腾空了也起不来它，不为它封那台；一台都放不下的不封组，排队那一刻打回 `memory-cap-at-maximum`。要卡的整片只在空卡凑得够它要的张数的台上算，卡被占着的台不为它封组。`/health` 判排头放不放得下与派活同一个判法（内存、编译目录占盘、CPU 闸、卡的张数）。组的账是这一组装过多少碎片：跑完的照算（装到界就封，整片才轮得到；跑完就减的话碎片一条接一条地来、组永远不封），封着组时插空起的不记进去；封组那一句另写这一组此刻还在跑的几条与它们的记账，跑完的跟着减。封着组时插空：排头整片在那台上照样空着的那一份是「配置的总上限 − 余量 − 排头的记账 − 挨着整片插空在跑的上限之和」（封组之前起的碎片不减：整片本来就等它们跑完），放得下的碎片不看时长就起（两台都封着组时没有预计用时的小任务靠这一条起，任务 8370）；会占排头那部分的，预计用时不超过那台在跑的尝试里最长的剩余用时（挡路时长）才起，没有预计用时的照旧封着；排头整片记满那台可用上限的，那台这一份是 0。预计用时：`scripts/crash/crash-point-replay.sh`、`.claude/gate.d/formal-verification.sh`、`.claude/gate.d/crates-mutation-replay.sh` 点了 `--item` 而不分片的，服务先起一条估算子任务（同一条入口接 `--list-items --with-keys`，读每项一行 `ITEM_PLAN item=<项名> to_judge=<要现判的条数> reusable=<能复用的条数>`），它有了结局才进队；每次跑完照日志里每项一行 `ITEM_DONE item=<项名> seconds=<秒> judged=<现判条数> reused=<复用条数> exit=<码>` 记每项「每现判一条几秒」与「每复用一条几秒」（记在服务自己的库里，不进 prune-server），预计用时是各项条数乘速率之和，缺一项速率就没有预计用时；估算没成的照没有预计用时排。别的命令的预计用时是同一命令键（与内存记录同一把键：路径、条目照词折过，`--check` 的格名照留）最近一次命令自己跑到退出的用时（退 0 与命令自己退非 0 的都算；被信号杀的、撞上限、超时、卡住被停、被挤掉、取消、跑满两小时这些服务判出或打断的不算）；分片的计划、各片与合并三步也是（每一步按自己的命令键取自己最近一次跑成的用时，各片按片号各一份；计划那一步不交每片的项，各片按项估做不到）。`task status <号>` 在预计用时后面写这个数是按命令、按项还是没有记录。提交满两小时的排到最前、先判；满两小时的整片照样封组等它、它自己不受交替挡，为它封着组的那台一条都不插、等腾空起它（用户：「一个任务都排了两个小时了 还要有任务插队 明显不合理」）；放不下时不挡后面、不留份，后面放得下的照队列先后起。排着的一律不留账。GPU 任务照多机配置里卡的先后挑（判法在 `task_server.py` 文件头「GPU」）；卡的台标签写多机配置 `HOSTS` 里那一台的地址，每项 `<地址>:<卡号>:<额度>`（例 `192.0.2.1:0:32G`；地址可以是 IPv6，从右边切两刀），台标签不在 `HOSTS` 里的判配置写坏，服务照 `HOSTS` 认出自己这一台、交给命令的 `GPU_CARDS_BY_PRIORITY` 是原样那一项；带上卡特性（`gpu`、`tests-on-a-real-gpu`，连同 `<包>/<名字>` 与 `--all-features`）的 cargo 只有 `test`、`run`、`bench` 跑编出来的代码、又不带 `--no-run` 的才要卡，`clippy`、`check`、`build`、`doc` 与 `--no-run` 只编不跑，不要卡、不清场。按参数才要卡的脚本在自己文件头前 60 行写一行 `# task-server-gpu-arguments: <词> [<词>…]`：bash 或 python3 起它、第一个非选项参数是其中一个词就要卡（`*` 是任何参数都要卡），写法在 `task_server_lib.py` 文件头「GPU」。两台走同一段判法（记账、放不放得下、卡住与 CPU、峰值、上限、编译目录、挑台、盘余量，盘余量两台同一个 50G 下限），按台不同的只在怎么连上、怎么量、怎么起那一台（`task_server.py` 的 `Machine`）。尝试正常结束之后单元里还留着命令起的孙进程的，服务停掉、结局说明写明；跑超过 30 分钟而失败、脚本又没在文件头写 `# task-server-resumable: <词>…` 声明续跑的，结局说明写「这条跑了 N 分钟、没有续跑：重交会从头跑」。

每次尝试的命令外面套 `scripts/hosts/capped.sh <每台默认重活线程数>`，线程一律是这个数，提交时不收；要卡的任务一律在最外层再套 `scripts/crash/gpu-quiesce-run.sh --`。内存包装、`capped.sh`、清场包装与 `multi-host-run.sh --peer` 只认服务直接套的那一层：服务给每一层设一个准入标记（值是 `<任务号>.<第几次>`），那一层读掉、不交给命令，任务里面的脚本再套一层当场拒，出路是删掉那一层，内存、线程、卡照读服务交的 `TASK_SERVER_MEMORY_CAP_BYTES`、`SINGLEFS_THREAD_CAP`、`GPU_CARDS_BY_PRIORITY`；会编的 cargo 另给这一次一个仓外的 `CARGO_TARGET_DIR`，有了结局服务就删（提交时设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1` 的留着，`wait` 末行带 `build=<目录>`，由提交它的会话 `task clean <号>` 删）；取消的在同一次取消里连现场一起删（下面「## 提交写法」的取消）。定额度只照服务自己库里的尝试记录（近几次跑成的峰值的最大值 × 1.5；不读内存包装的峰值表；命令键先剥掉 `env` 与多机配置路径、`LANG` 这类不改跑法的变量前缀），没有记录的先记 4G（只判红过的不低于判红那一次的峰值 × 1.5），有记录的照记录 × 1.5 记上限与记账、不抬到 4G（钉值照旧），撞了乘 1.5 重排（重排的也不兜到 4G），乘 1.5 会到这台总上限的不再重排，有总上限更大的另一台就换过去，没有就记 `memory-cap-at-maximum` 打回提交它的会话，`next=` 与发给会话的正文写撞了哪台、到了多少、峰值多少，修改意见是核实这一趟的范围（点的项、测试目标）、分片与同时起的个数，收小或分片再交；定额度、编译族与兄弟片只读峰值口径一栏写着「进程内存」的尝试记录，没有这一栏的旧记录（含页缓存）不读；算这台还放得下多少只算进程用的内存（`anon` 加 `shmem`），页缓存与内核挂在 slice 上的不算；起来之后先宽限 10 分钟不判；之后日志满 10 分钟没涨、这段时间 CPU 也几乎没涨（平均不到一个核的 1%）的服务停掉、记 `stalled`，读不到 CPU 不当没涨；日志与 CPU 都满 10 分钟不涨、而这段时间每一轮都有进程在 D 状态（不可中断的 IO 等待：本机读这一次尝试各进程 `/proc/<号>/stat` 的状态，第二台读 `multi-host-run.sh` 每轮打的 `MULTI_HOST_RUN_PEER_IO_WAIT poll=<轮> blocked=<D 状态的进程数> read=<yes|no>`，这一行不算日志在涨）的是卡在 IO 上，不记 `stalled`、不停，有一轮没有 D 状态的进程或读不到的照原判法判（任务 8608、8609、8611、8619 在第二台上等本机导出的 NFS 时被当成卡住停掉）；要卡的任务、这一次尝试分到了卡的，日志与 CPU 都满 10 分钟不涨、而这段时间每一轮它那张卡上都有这一次尝试的计算进程（本机读 `nvidia-smi -i <序号>` 的计算进程里有没有这一次尝试的子孙进程，第二台与量不到的当有）、卡的利用率有一轮不为 0 的（本机同一个 `nvidia-smi -i <序号>` 读 `utilization.gpu`；第二台与读不到的照有计算进程算，依据写「利用率读不到」）是 GPU 在算，同样不记 `stalled`、不停；卡上一直有它的计算进程、显存占着而利用率每一轮都是 0 的卡没在干活，照原判法停、记 `stalled`（任务 9341：显存占着 14129 MiB、利用率 0%，最后几条用例半小时不动）；这种占着卡不干活的，「CPU 也几乎没涨」的门槛提到命令自己的 CPU 这一窗口平均不到 1 个核（CPU 上轮询死等的照停，说明写日志多久没涨、卡的利用率与命令自己的 CPU 三样读数；不少于 1 个核的是在 CPU 上算，照跑）；有一轮卡上没有它的计算进程的、没分到卡的照原判法判（任务 8519、8520 两条单条 GPU 对拍在本机分到卡、GPU 在算时被当成卡住停掉；用户定 GPU 任务「不自己终止，告诉负责的对话 自己检查」）；跑过记录用时 5 倍、日志又满 10 分钟没涨的不看 CPU 也停掉、记 `stalled`（死循环也停），没有记录用时的不判这一条；`stalled` 不重试，打回提交它的会话；判卡住那一刻这一次进程用的内存（`anon` 加 `shmem`，同上面算余量的口径；页缓存撑满上限时内核回收它、不杀进程，不算）此刻或峰值已到上限 98% 的是顶在上限上卡住，照样停掉，但记 `memory-cap-hit`、乘 1.5 重排（钉值的打回 `memory-cap-at-maximum`），说明写「顶在上限上卡住」；已跑多久、记录用时、在用核数只写进 `task status`。记的内存峰值两台都只算进程用的内存：每一轮采样读内存包装那一层 scope 的 `memory.stat` 的 `anon` 加 `shmem`、留见过的最大（任务 8853 编验证包时进程不到 1G，页缓存把 scope 的 `memory.peak` 撑到上限 50176M）；比一轮还短、一次都没采到的才照包装退出时读的 `memory.peak`。第二台那一层的内存峰值照 `multi-host-run.sh` 打的 `MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<字节>` 记进那一次尝试，同本机一样进定额度，不打的记没量到。编译目录占盘也按命令记峰值（本机量服务给的编译目录，第二台照 `MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<字节>`），派到一台之前那台的盘余量减去在跑的还要涨的，要放得下这条命令的占盘峰值加 50G 下限，放不下就排着、排队原因点名「编译目录占盘」。命令自己退 75（prune-server 不可用）记 `prune-server-unavailable`、`wait` 退 75、不重排，服务另起一次 `python3 prune-server/control check`，不通写告警，不起、不停 prune-server。命令写成 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>` 的，算命令键、认登记表行照 `--` 后面的原命令（`--server-binary` 的路径不进键），命令键前面另接「〈task-server 临时 prune-server〉」：同一个 scope 里多起一份 prune-server，不与不包的那一条共用记录；包装照旧整条执行。

准入按上限记账：在跑的与新起的都按上限记（收到那台总上限减那台此刻的余量为止），每台在跑任务的上限之和不超过那台 slice 的总上限减那台此刻的余量，还没涨到量的那一截按「上限 − 此刻回收不掉的用量」算；两台同一段：用量由读那一台余量的那一次（第二台是同一次 ssh）按每条尝试的 scope 量回来（scope 照里面进程环境的 `TASK_SERVER_ATTEMPT_LABEL` 对到尝试上，两台都给），报告里没有那一条的 scope 记整份上限、不记 0，第二台那一层内存包装不另记账。余量是那台此刻 `singlefs-heavy.slice` 里回收不掉、又不归任何在跑任务的那一截（slice 自己记着、哪个子 cgroup 里都没有的，量法在 `scripts/hosts/memory_peaks_lib.py` 文件头「余量」），每次读两台的余量时各量各的，按 1 GiB 向上取整（量到 1.01G、1.1G 记 2G，2.1G 记 3G，0.7G 记 1G，恰好整数的不进位）；它不是常量，也不在配置里。哪一台量不到余量，那一台这一轮不派，排队原因写哪一台与原因；能派的每一台都量不到的不起、记 `run-condition-unmet`（`wait` 退 78），不退回固定的数。每次尝试经环境 `TASK_SERVER_MEMORY_CAP_BYTES` 把这一次的上限（字节，十进制）交给命令，本机与第二台都给。`python3 task-server/control pin-cap <命令…>` 给一条命令钉上限（`unpin-cap <命令…>` 撤；经服务写库，服务在跑时也能用）：钉了的命令上限与记账都取多机配置 `SLICE_MEMORY_CAP_BY_HOST` 里派到的那一台那一项减那台此刻的余量（`<地址>=<值>` 空白分隔，写的是那台 slice 的总上限，配多少就是多少；每次现读，不记字面的数；那一项不比那台的余量大的，`pin-cap` 当场拒、出路是把那一项改大），学到的峰值不顶掉它，撞了不放大、记 `memory-cap-at-maximum`，`task status` 写明是钉值；钉了而配置的 `SLICE_MEMORY_CAP_BY_HOST` 读不到（没有、写法不对、少一台、多了不在 `HOSTS` 里的地址、文件读不了）的不起、记 `run-condition-unmet`（`wait` 退 78），`wait` 的理由行写读不到的原因与出路（改对配置或 `unpin-cap`）。疑似卡住只通知：宽限过后整个单元的 CPU 满 10 分钟平均不到一个核的 1%（要卡的另要这段时间它那张卡上没有它的计算负载，量不到当有）的，不管日志涨不涨，服务不停它：`task status` 写「疑似卡住」与依据、记一条告警，`wait` 打一行 `TASK_SERVER_SUSPECT task=<号> …`、退 231，任务照跑，再起 `wait` 接着等（同一段只报一次，CPU 又涨了算这一段结束）。日志满 10 分钟没涨、此刻内存不低于这一次上限的 98% 的，CPU 在涨也停掉（在回收里打转不算在算；任务 7723 贴着 6144M 的上限这样跑了 37775 秒）、记 `memory-cap-hit`、乘 1.5 重排；两台同一段判，本机读外层 scope 的 `memory.current`，第二台读 `multi-host-run.sh` 每轮打的 `MULTI_HOST_RUN_PEER_MEMORY` 一行（这一行与 `MULTI_HOST_RUN_PEER_CPU` 一样不算日志在涨）。点到 `.claude/gate.d/` 的门禁派得到第二台（它跑的就是这棵共享的树）；带判定库而登记表 `.claude/gate.d/verdict-library-scripts.tsv` 那一行第三列还没写 prune-server 的，提交时就拒（下面「## 提交写法」）。`python3 scripts/experiment-apparatus/device-code-key.py run … --feature <值>` 的特性值带 gpu 的也要卡。`/health` 只做库写入试探与排队线程心跳，排头放得下却不起由排队线程每一轮算好、`/health` 只读。

跑满两小时打回：提交时的门禁之外，一条任务各次尝试在跑的时长累加（排队等的不算；撞了内存上限、照大一档重来的计时归零，只从最后一次撞上限之后那一次起算，撞几次都一样；分片的各片各自累计）满 120 分钟，服务停掉它、结局记 `over-two-hours`（`wait` 退 242），推给会话的正文写命令、范围声明、累计与各次用时、日志末几行，请它核两件事：范围对不对（点名范围：`--item`、`--check` 少几格、点名测试目标；给补丁目录；或照服务分片；整轮的留到提交时）、这条是不是确实要跑这么久。不对的改了范围照常 `task submit`；对的照推送里那一整行 `task-server/task requeue <号> --long-run-confirmed --items <N> --tasks <M> --why '<为什么要跑这么久、范围为什么是这些>'`：只许提交它的会话、只对结局是 `over-two-hours` 的号，两个数与 `task confirm` 同一个核法（对不上退 3、打服务算的确认单），`--why` 去掉空白至少 12 个字，同一个号只许一回；核过就以原命令、原范围声明新进队尾（排位按 requeue 那一刻），记「长跑已确认」，不再按两小时打回，别的判法照旧，`task status`、列表、推送与日志头打一行「长跑已确认（原号 <n>）：<why>」。原号不自动重排。豁免只有提交时的门禁：提交时的六道门禁（`crash-point-replay`、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation）或 `scripts/gate/gate-staged.sh`，不带 `--check` 与 `--item`，范围声明的「为什么」写了「提交前」；带 `--check` 的门禁、`merge-and-verify`、入口脚本、实验都不豁免。

整包测试要确认标识：`cargo test` 不点名 `--test`、`--bin`、带测试名的 `--lib`，或带 `--lib --tests`、`--workspace`；`.claude/gate.d/crash-point-replay.sh --check` 把 crash-case-replay、feature-gated-tests、product-gpu-real-card-tests 三格全点；不给补丁目录的 `scripts/gate/merge-and-verify.sh`——这几种是整包测试，没带 `task submit --whole-package-confirmed <确认文件>` 的提交时当场拒（退 3，异常表 `whole-package-unconfirmed`），提交时的门禁豁免。确认文件照 `.claude/agent-common.md`「日常开发的测试不跑整个包」写三遍确认（①改到的文件与函数、②引到它们的测试目标与函数及 grep 命中、③为什么只点名②不够），放共享根下、写绝对路径；服务只判形式（读得到、三段都在、每段去掉空白至少 12 个字），不合的退 3、点名缺哪一段。核过照收，`task status`、列表、推送与日志头打一行「整包已确认：<文件>」；带它照样受两小时限，打回之后 requeue 新进的那一条两种标识都带。

撞上限那一刻的内存明细：两台的内存包装判出撞了这一条自己的上限（退 250）那一刻往日志打一行 `RUN_WITH_MEMORY_CAP_OOM_STAT anon=… file=… file_dirty=… file_writeback=… shmem=… oom_kill=…`（`scripts/hosts/run-with-memory-cap.sh` 文件头「撞上限那一刻的内存明细」），服务把它换成一句抄进那一次尝试的结局说明与推给会话的正文，写明大头是进程自己的内存（anon）还是页缓存（file，含没写回的脏页）；只记，不改判法。

## NFS 共享

**为什么要。** 用户定的原则：「所有的脚本必须多机跑。抛出的结果需要多机都能读取。如果不行，要么复制资源，要么共享文件。」NFS 是共享文件这一条路：本机导出若干目录，第二台挂到同一个绝对路径上，一台写出的结果另一台直接读到，任务引用的路径在两台上指同一份东西。用户 2026-11-01 又定：「NFS 的配置要写在 task-server 的 readme 中。不要遗漏了。这部分也要作为检测项目。没有 nfs 不启动。要求用户配置。配置相关来自多机配置的配置文件中 不允许增加额外的配置文件。」所以服务不替用户配 NFS，配置只有多机配置 `multi-host.env` 里的一个键，没配好服务就不起。

**共享哪几段。** 三段：仓本身（主工作树的根：第二台在这棵活树里跑，不拷树）、共享数据根 `~/.local/share/singlefs`（工作根与分片的产物在它下面）、会话记录目录 `~/.claude/projects`；都在本机上取绝对路径，第二台挂到同一个绝对路径（第二台的家目录可以是另一个，挂载点照本机的路径建）。

**本机导出。** 本机装 `nfs-kernel-server`，在 `/etc/exports` 里给每个共享的根写一行，只导出给第二台的有线直连段地址（真地址是私有值，只写在本机的这个文件里），再让导出生效并开机自启：

```bash
# /etc/exports，每个根一行
<根> <第二台的有线直连段地址>(rw,sync,no_subtree_check)
sudo exportfs -ra
sudo systemctl enable --now nfs-server
```

**第二台挂载。** 第二台装 `nfs-common`，在与本机相同的绝对路径上建目录，把父目录的属主改成与本机相同的 uid，在 `/etc/fstab` 里给每个根写一行，再让 systemd 重读并挂上：

```bash
# /etc/fstab，每个根一行
<本机的有线直连段地址>:<根> <根> nfs4 rw,hard,nofail,_netdev,x-systemd.automount,x-systemd.mount-timeout=30 0 0
sudo mkdir -p <根>
sudo systemctl daemon-reload
ls <根>                     # 走进去触发自动挂载
```

**本机重启之后自己恢复。** fstab 那一行带 `x-systemd.automount`、不带 `x-systemd.idle-timeout`。`x-systemd.automount` 让 systemd 在这个根上放一个自动挂载点（`.automount` 单元），它不随 NFS 挂载本身的成败失效：本机重启、导出不在的那段时间里挂载单元会变成 `failed`，下一次有进程走进这个根，systemd 就再起一次挂载，本机的 `nfs-server` 回来之后就重新挂上，不用人去 `reset-failed`、`start`。不带 `x-systemd.idle-timeout`：挂上之后一直挂着，不会闲着就卸掉。`hard`（NFS 缺省）让本机重启那段时间里在读写的进程等着、不报错。服务的挂载判法不看 `mountpoint -q`（自动挂载点闲着时那里只是一层 autofs 的占位，`mountpoint -q` 照样退 0），在第二台上 `stat -f -c %T -- <根>/.`：走进目录触发挂载，读到 `nfs` 才过。挂上之后在第二台上 `git -C <仓的根> rev-parse HEAD` 读得到与本机相同的提交，就是仓那一段通了。

**uid。** 两台上用户的 uid、gid 要相同；不同就在导出项的括号里加 `all_squash,anonuid=<本机上用户的 uid>,anongid=<本机上用户的 gid>`，让第二台写进来的文件都落成本机这个用户的。

**多机配置写哪几键。** 在仓根的 `multi-host.env` 里写三个键。一是 `NFS_SHARED_ROOTS=<绝对路径>[:<绝对路径>…]`，就是上面导出与挂载的那几个根——仓本身（主工作树的根）、共享数据根 `~/.local/share/singlefs`（工作根在它下面）、会话记录目录 `~/.claude/projects`，都写本机上的绝对路径：冒号分隔，至少一段，每段以 `/` 起头、不以 `/` 结尾、不含空白；`ENABLE_ACROSS_MACHINES` 写 0 也要写。模板在 `multi-host.env.example`，不另设配置文件。多机配置判法 `scripts/crash/crash-point-replay-shard-configuration-check.sh` 判写法，在本机上另判每段是已在的目录，不合的退 1 并点名第几段。二是 `SHARED_DATA_ROOT=<绝对路径>`：共享数据根（工作根在它下面），本机上的绝对路径，落在 `NFS_SHARED_ROOTS` 某一段下；服务照 `.claude/hooks/lib_work_root.py` 取它（环境里设了 `SINGLEFS_DATA_ROOT` 照环境），`control install`、`start`、`check` 只看这一行，没写、不在共享根下都退 78、出路点名往 `multi-host.env` 加这一行，不补缺省。三是 `HOSTS=<本机的 IP> <第二台的 IP>`：各台有线直连段的地址，空格分隔；哪一台是自己照本机网卡地址认（`ip -o addr show`），第二台是另一项，ssh 直接连它，用户名靠各台的 `~/.ssh/config`。第二台的地址只经共用取法 `scripts/hosts/multi_host_addresses.py` 取，它整份判过多机配置才给地址：缺键、判法认不得的键、写坏的行都取不到，服务不起、`control start` 退 78。两台各有各的值的写进按台的三个键 `CARGO_BIN_DIRECTORY_BY_HOST`、`BUILD_ROOT_BY_HOST`、`SLICE_MEMORY_CAP_BY_HOST`：值是空白分隔的 `<地址>=<值>`，每项从第一个 `=` 切，`HOSTS` 里每一台恰好一项、地址都在 `HOSTS` 里，要哪一台的就按那一台的地址取；服务读的是 `SLICE_MEMORY_CAP_BY_HOST`（钉值），少一台、多一个不在 `HOSTS` 里的地址都判写坏。分片的各片经 `TASK_SERVER_SHARD_HOST` 拿到它所在那一台在 `HOSTS` 里的地址。

**服务起来时判什么。** 四项，任何一项不过服务不起、退 78，点名是哪一项并指向「## NFS 共享」：

- 配置：`NFS_SHARED_ROOTS` 在，每段是本机上的目录；`HOSTS` 认得出本机与第二台（下面两项照它连第二台）；
- 导出：本机 `/var/lib/nfs/etab` 里每段都有导出行；
- 挂载：第二台上每段 `stat -f -c %T -- <根>/.` 读到 `nfs`（走进去触发自动挂载）；连第二台的 ssh 一次超时先当「慢」，隔 5 秒复查一次，复查通了只在服务日志记一行「第二台慢（超时、复查通了）」、不推提醒、照派第二台，复查还不通才算不过（「连不上第二台」）；
- 读写：两个方向各写一个探针文件，对方读得到（本机写、第二台读，第二台写、本机读）。

`task-server/control check` 每次现判这四项（它自己打出结论，不改服务的 `/health`）；`/health` 的 `nfs` 栏同样报这四项，其中「读写」那一项（两向探针）是服务起来时那一次探针的结论，之后不再重做；服务起来之后只复核前三项、每分钟一次，转为不过的，服务直接发消息给名下有任务在排、在跑的会话（见「## 发消息给会话」）。

**不过时怎么办。**

- 配置不过：照「多机配置写哪一键」补键、改写法，目录没有就先建。
- 导出不过：补 `/etc/exports` 缺的行，`sudo exportfs -ra`；服务没起就 `sudo systemctl enable --now nfs-server`。
- 挂载不过：第二台照上面的写法补或改 `/etc/fstab` 那一行，`sudo systemctl daemon-reload`；挂载单元成了 `failed` 的 `sudo systemctl reset-failed "$(systemd-escape -p --suffix=mount <根>)"`，再 `ls <根>` 触发挂上；挂不上先查有线直连段通不通、本机导出的客户端地址是不是第二台。
- 读写不过：多半是两台 uid、gid 对不上，或导出项没写 `rw`，照「uid」一段办。

**怎么核。** 三样，各在注释写的那一台上跑：

```bash
stat -f -c %T -- <根>/.                   # 在第二台上跑：打 nfs 就是这个根挂上了
cat /var/lib/nfs/etab                      # 在本机上跑：每个根都有导出行，客户端是第二台
echo probe > <根>/nfs-check-by-hand        # 在本机上跑：本机写
cat <根>/nfs-check-by-hand                 # 在第二台上跑：第二台读得到 probe
echo probe > <根>/nfs-check-peer           # 在第二台上跑：第二台写
cat <根>/nfs-check-peer                    # 在本机上跑：本机读得到 probe
rm <根>/nfs-check-by-hand <根>/nfs-check-peer   # 核完删掉两个文件
```

## 提交写法

```bash
task-server/task submit -- <命令…>                 # 打出任务号与确认单，任务待确认、不进队；环境里的 SINGLEFS_RUN_SCOPE 这类白名单变量照带；读不到会话的收件地址当场拒
task-server/task submit --whole-package-confirmed <确认文件> -- <命令…>  # 整包测试带三遍确认的那份文件（上面「整包测试要确认标识」）；不带的整包测试当场拒
task-server/task requeue <号> --long-run-confirmed --items <N> --tasks <M> --why '<…>'  # 跑满两小时被打回的那一条确认确是要跑这么久：原命令新进队尾、记长跑已确认（上面「跑满两小时打回」）
task-server/task confirm <号> --items <N> --tasks <M>  # 照确认单核过范围之后确认：两个数都与服务此刻算的相同才进队（照提交时刻排位置），对不上退 3、打服务算的确认单
task-server/task wait <号>                         # 取结局：完成时退出、末行 TASK_SERVER_RESULT …，退出码是任务的；记着会话收件地址的任务不必为了被叫醒起它（见「## 发消息给会话」）
task-server/task status [<号>]                      # 这个会话待确认、在排、在跑、完成了没人取的；给号的待确认那一条打此刻现算的确认单；排着的带「在等：…」，在跑的带已跑多久、日志多久没涨
task-server/task cancel <号>                        # 只许提交它的会话；回读到停了，同一次取消里删这条任务的编译目录与现场，打删了几份、删前删后多大
task-server/task log <号>                           # 最后一次尝试的日志（最后 200 行；--all 整份；--attempt <第几次> 取那一次；任务或那一次不在退 4）
task-server/task clean <号>                         # 删跑完的任务留下的编译目录与现场（只许提交它的会话、有了结局才删；没设 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的编译目录有结局就删了、现场在 wait 取走之后一小时删；第二台那一趟的编译目录 multi-host-run.sh 自己删；取消的不用它）
```

交任务的二次确认：`task submit` 收下的任务先记待确认（`awaiting-confirmation`），不进队、不建子任务、不起尝试；标准输出接着打服务回的确认单：整条命令、`SINGLEFS_RUN_SCOPE` 的四段原文、服务从命令里认出的项数 N（`--item`、`--check` 的每一格、点名的 `--test`、`--bin`、`--lib` 与 `--exact` 之后的测试名各几个；一个都没点的写「整包、不指定范围」）、这个会话此刻在排在跑的条数 M 与其中命令键相同或点的项交叠的逐条列号、预测内存与预计用时与记录来源、交的时候就判得出的异常（预算超过或贴着整台、按给的上限开并行，带那一种的修改意见），末行 `TASK_SERVER confirm-needed task=<号> items=<N> tasks=<M> timeout=<秒>`。核过范围对了写 `task-server/task confirm <号> --items <N> --tasks <M>`：两个数都与服务此刻现算的相同才进队，任务号是提交时刻的、照它排位置；对不上的退 3、打服务此刻算的确认单（这一会儿同会话别的任务起了或收了，数会变）。分片的父任务确认一次，子任务由服务建、不再要确认。满 30 分钟没确认的服务取消（结局 `cancelled`，说明以「没确认，已取消」起头），照结局发给会话。只有提交它的会话能确认；子 agent 不交任务，也不确认。

取消的不留产物（用户：「取消的任务要自己删掉产物 不要残留」「这个加入流程中 让task server去做」）：`task cancel` 回读到单元停了，服务在同一次取消里删这条任务（分片与估算的连同子任务）每一次尝试的编译目录与现场目录（设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1` 的也删）；派到第二台的，第二台本地的 `builds/<标签>/` 与 `runs/` 下进程号、退出码、输出三个文件由 `multi-host-run.sh` 的 ⑥ 删，服务照那一次尝试日志里它打的「⑥ 删了第二台这一趟的本地目录 …（回读不在了）」认，没有这一行而清理回执还是 `not-started`、日志里也没有建了那一趟本地目录之后才打的行的，是还没建，不算没删成，上一次留着的经 `--remove-build-directory` 删。删前删后的大小逐份打在 `cancel` 的输出里、写进那条任务最后一次尝试的日志（第二台本地盘上的那一份服务这一侧量不到，照写）；回读不在了才算删了，没删成的逐份点名、`cancel` 退 5。

提交时警告（用户：「绕过的 那个要做成提醒 谁做错了严厉警告」）：照收照跑，服务当场打警告，进提交回复（客户端打到标准错误）、这条任务每一次尝试的日志头与 `status`。头一行「✗ 严厉警告：会话 <号> 交的任务 <号> 绕过了服务给的编译目录，取仓根 target/ 下的二进制（…）…」，第二行「→ 怎么办：…」。判的是：命令里的词落在仓根 `target/` 下，或命令位置上起的本仓脚本正文里有指向仓根 `target/` 的路径字面（前面接仓根变量、`$(dirname …)/../..`、`$(git rev-parse --show-toplevel)`，接 `target/release/`、`target/debug/`；在仓根起的另认裸写的）。服务给的编译目录、调用方给的 `CARGO_TARGET_DIR`、草稿与临时目录下的 `target/`、注释、弄坏开关那一支与自证那一支（带 `--selftest` 起的、名字带 selftest 或以 cells 收尾的函数体）不算。判法在 `task_server.py` 文件头 ②「提交时警告」①。另一种（同一节 ②）：交任务时会话名下有任务的结局满 30 分钟没交给它（服务发的消息没送到，也没起 `wait`），照收照跑，同样进三处，头一行「✗ 严厉警告：会话 <号> 有 <N> 条任务跑完了没人取（<号>、…）：…」，第二行「→ 怎么办：task-server/task wait <号> 取走（…）…」；没有会话的人手任务不判。

`wait` 自己报任务要处置的八种情形（用户在弹窗里选「task wait 自己报」）：一是排队过久，排满 `QUEUE_ATTENTION_MINUTES`（30 分钟）；二是跑到记录用时的 `OVERRUN_ATTENTION_FACTOR`（3）倍，日志在涨时服务不停它，靠这一条叫人；三是换卡重排，清场判出卡坏、服务换别的卡重排，结局可能是 `succeeded`；四是清场没收尾（`cleanup-pending`），命令已退出而清场回读没收干净，服务保留资源占用；五是分片的父任务推不动（`shard-advance-failed`）；六是日志长期不涨而服务没停它（`quiet-log-busy`），日志满 `QUIET_ATTENTION_MINUTES`（30 分钟）没涨、服务因为 CPU 在涨或读不到 CPU 而没停它，有没有记录用时都报；按卡在 IO 上或卡上有它的计算进程免停的不报这一条，由 `io-wait`、`gpu-busy-quiet` 报；七是卡在 IO 上而服务没停它（`io-wait`），日志没涨的每一轮都有进程在 D 状态、连着满 `IO_WAIT_ATTENTION_MINUTES`（30 分钟），依据写最近一轮 D 状态的进程数与本机此刻 `/proc/pressure/io` 的 some avg60；八是卡上有它的计算进程而服务没停它（`gpu-busy-quiet`），分到了卡、日志与 CPU 都不涨而每一轮卡上都有这一次尝试的计算进程、卡的利用率有一轮不为 0 或读不到、连着满 `GPU_BUSY_QUIET_ATTENTION_MINUTES`（30 分钟），依据写卡的键与卡的利用率（读不到的写「利用率读不到」）；利用率一个窗口里都是 0 的照卡住停，不报这一条。另有第九种只报有结局的任务：改主工作区的命令（`apply-writer-patch.py`、`hook-swap.py`、`stage-mine.py`、不带 `--skip-apply` 的 `merge-and-verify.sh`）以撞上限、被挤掉、显存不够、第二台没跑成、撞顶、取消、卡住被停、对不回来或被信号杀收尾（`workspace-write-interrupted`），可能写了一半，`next=` 点名要核的文件或补丁目录（用户：「这个问题很严重 因为可能会导致补丁没有正常写入」）；`apply-writer-patch.py` 有结局文件 `apply-outcome.tsv` 的，服务照它的 `--read-outcome` 判，打全了、已退回的不报，带 `--dry-run` 的不报。另有三种（共十二种；用户：「既然加了给对应对话发消息的功能 就就该把异常和修改意见交给对应的对话」），只提醒、照排照跑，依据写量到的数、`next=` 写修改意见：十是预算贴着整台（`oversized-budget`），排队那一刻判出它在某一台上的记账不小于那台的整片界（可用上限的 85%）而不超过可用上限，依据写各台的记账、可用上限与记账从哪来（记录峰值几次、编译族、钉值），钉了上限的照样报，修改意见是核实范围、分片与同时起的个数；记账比能派的每一台的可用上限都大的不进队空等，排队那一刻就打回 `memory-cap-at-maximum`、一次都不起；十一是跑得久而核没用起来（`long-run-idle-cores`），一次尝试跑满一小时、到此刻平均在用核数不到给的线程数的四分之一（任务 8856 给 31 线程平均只用 8.881 核），每次尝试只报一次，修改意见是核实脚本是否串行、范围是否点多；十二是按给的上限开并行（`fills-any-cap`），同一命令键最近三次跑成的峰值都不低于各自上限的 90%（撞了上限的那几次不算：撞的那一次峰值本来就贴着上限，照上限一档档撞上去的不是按上限开并行），修改意见是「这条命令按给的上限开并行，核实」。每一种（连同提交时拒、提交时警告、各种结局、疑似卡住与结局没人取）怎么判、动作、修改意见与没处置时怎么办，只照 `task_server_lib.py` 文件头「异常表」那一行：`next=` 的头一段就是那一行的修改意见，点名这一种要核的东西（例：`io-wait` 核那台的盘与 NFS、`/proc/pressure/io`；`cleanup-pending` 核留下的进程命令行、脚本有没有不等子进程就退），后面接这一回的具体数。服务把这条任务（分片的连同子任务）的这几样记成「要处置」的提醒，正在等它的 `task wait <号>` 读到一条新提醒就打一行 `TASK_SERVER_ATTENTION task=<号> type=needs-attention kind=<种类> tier=<档> basis=<依据的数> why=<原因> next=<出路>`，然后退 230：任务照旧在排或在跑，服务没动它，按 `next=` 处置，再起 `wait` 接着等。任务已有结局的（换卡重排跑成）这一行打在 `TASK_SERVER_REASON` 之前，退码仍是任务的。同一条任务同一种提醒、同一档只叫醒一次，重起 `wait` 不再叫，档跨过下一档（阈值的两倍、四倍……）才再叫；报过的档记进服务的库，服务重启后照库认，同一档不再报；排队过久那一种，排队原因还没判出来（服务刚重启、这一轮还没判到它）的那一轮不报，等判出来那一轮再报。这几种提醒服务另直接发消息给提交它的会话（见「## 发消息给会话」），与 `wait` 同一份记账：发过的 `wait` 不再报，`wait` 报过的不再发。服务健康不归 `wait`：服务自己盯，同样发消息。

命令里不套包装、不写资源参数：`run-with-memory-cap.sh <上限>`、`host-dispatch.py … --`、`multi-host-run.sh … --`、`nice`、`timeout`、`capped.sh`、`two-logs.sh`、`gpu-quiesce-run.sh` 与脚本自己的 `--memory`、`--memory-cap`、`--threads`、`--peer`、`--local-only`、`SINGLEFS_THREAD_CAP`：这几样由服务做，写了当场拒并点名去掉哪一样。白名单外的环境变量当场拒并点名；命令真要它的写成 `task-server/task submit -- env <名>=<值> <命令…>`。退役的变量（`SINGLEFS_HEAVY_TESTS`、`GATE_MUTATION_START_OVER`）设在环境里、写在命令里（`env` 后面与命令前面的 `名=值`）都当场拒，拒绝说明点名它、写出路；名单是 `task_server_lib.py` 的 `RETIRED_VARIABLES`。

派不到第二台的三种提交时当场拒（用户：「改成提交时当场拒（推荐）：不再悄悄改派本机：碰到这三种就在提交时报错、点名是哪个路径或哪一族、出路写怎么改，任务不跑。」）：`task submit` 退 3，打那一句与出路，任务不进队、不跑。一是工作目录与命令里的绝对路径（只认整个 shell 词：以 `/` 起头的整个词、`名字=绝对路径` 与 `--选项=绝对路径` 的值、`bash -c` 脚本词照 shell 词切开之后的每一个；别的词中间夹着的 `/`（参数值里的「（接的是 /）」这类）不算）有不在多机配置 `NFS_SHARED_ROOTS` 任何一段下的，点名那个路径，出路是改放某一段下；`/dev/null`、`/usr/`、`/bin/` 这类两台相同的系统路径不算；`/tmp` 下的同样拒，那一句另说重启会清 `/tmp`。二是交给命令的 `SINGLEFS_DATA_ROOT`、`SINGLEFS_WORK_ROOT` 有不在共享根下的，点名变量与值，出路是配多机配置。三是命令点到登记表 `.claude/gate.d/verdict-library-scripts.tsv` 里带判定库的一行、而那一行第三列还没写 `prune-server` 的，点名那一族，出路是那一族接进 prune-server、把第三列翻成 `prune-server`；登记表读不了、写坏了的同样拒。

临时 prune-server 的包装（`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>`）只给脚本自证用（用户：「旧脚本不兼容 我再说一次 如果错了让 对方重新提交」）：剥出的原命令里有会编的 cargo、或要卡的（认法与服务定编译目录、分卡的是同一套），提交时照上面同一种形态当场拒：`task submit` 退 3，打那一句（点名会编还是要卡）与出路（去掉包装重交），任务不进队、不跑。剥开只用来判，收下的照旧整条执行；编译目录、分卡与清场只照外层那一条给，服务不剥包装去迁就。不带 `--server-binary`（命令前的赋值与任务环境里也没给 `PRUNE_SERVER_BINARY`）的包装自己要在服务给的编译目录里现编 prune-server，服务把外层这一条按会编的算：派本机、派第二台都给编译目录；带了 `--server-binary` 的不编、不给。验证包的 `cargo test` 对真服务判、不套这层包装。

不点名包的全仓编译提交时当场拒（用户：「8105打回 让他重新写重新确认范围」；任务 8105 不带 `-p` 的 `cargo build --offline --all-targets` 预测 49.0 GiB，第二台为它封组腾空）：提交的这一条在命令位置上直接起的、会编的 cargo（`build`、`check`、`clippy`、`test`、`run`、`bench`、`doc` 与它们的单字母别名；`bash -c` 与 `cd` 跟着走）带 `--workspace` / `--all` 的拒；没带 `-p` / `--package`、照 cargo 找清单的走法（起的目录往上第一份 `Cargo.toml`）落到虚拟清单（只有 `[workspace]`、没有 `[package]`，仓根那一份就是）的拒，起的目录认不出、清单读不了的同样拒。`task submit` 退 3，打那一句（哪一条 cargo、在哪个目录起、照哪一份清单）与出路：点名要编的包 `-p <包>`（验证包 `-p singlefs-verification`），整仓要跑的走 `scripts/gate/merge-and-verify.sh` 或提交时的门禁。落到包自己的清单的（`scripts/experiment-apparatus/`、`prune-server/` 这类自成工作区的包，或在一个成员的目录里起）只编那一个包，照收；入口脚本与门禁里面起的 cargo 不判（服务只看提交的那一条）；`cargo fmt`、`metadata`、`tree`、`clean` 不编，不判。

## 发消息给会话

用户：「不是你通知 而是 task-server 通知发起的对话」「包括报错 长时间没有响应 都可以这样通知」。`task submit` 在会话的 Bash 里跑，读环境里的 `CLAUDE_CODE_MESSAGING_SOCKET`（会话的收件 socket）与 `CLAUDE_CODE_MESSAGING_TOKEN`（会话令牌），随任务交给服务，服务记在任务上；令牌不进日志、`status` 与 `/tasks/<号>` 的回话。读不到的当场拒（退 3），出路是从 Claude Code 会话的 Bash 里交（子 agent 不交任务）。收件 socket 不照交任务时的那一个：每次发之前服务按任务记的会话号读会话登记（`~/.claude/sessions/*.json`，一份一个进程，只读 `sessionId`、`messagingSocketPath`、`pid`），取会话号相同、进程还活着的那几份里启动最晚的一份的 `messagingSocketPath`；令牌取交任务时带的。会话重启换了 socket 的，照新的发；找不到的（会话已关）照下面「发不出去」写告警。

服务在这几件事上往那个 socket 发一条消息：

- 任务有了结局（`wait` 会打 `TASK_SERVER_RESULT` 的每一种）：只发一次；发出去了服务记成已取（同 `wait` 取走），会话已经用 `wait` 取了的不发。
- 要处置的提醒（上面「## 提交写法」那十二种）与疑似卡住：同一条任务同一种提醒同一档只发一次，与 `wait` 同一份记账。时长判的照档每翻一倍升一档再发；档不随时长涨的（`oversized-budget`、`fills-any-cap`、`long-run-idle-cores`、`cleanup-pending`、`card-replaced`、`shard-advance-failed`）同一档发过之后满 30 分钟还在的再发一回、之后每翻一倍再发，第一行写「没处置、第 <回> 回再推」，正文另写之前推过什么、此刻还差什么。
- 结局没人取：父任务的结局满 30 分钟还没交给提交它的会话（发结局的那一条没送到，也没起 `wait`）的，再推一回、之后每翻一倍再推，第一行「task-server：任务 <号> 结局还没人取（…）」，推出去了记成已取。
- 交了没确认、满 30 分钟被服务取消的：照结局发，正文带「没确认，已取消」。
- 服务健康：库写不进、派活判排头能起却一直没起成、看进展报放得下却一直没起、服务的单元不在、NFS 共享前三项不过、第二台用不了满 180 秒。发给每个名下有任务在排、在跑的会话，同一样对同一个会话只发一次，好了再坏再发。服务自己挂了发不了：`task wait` 连续连不上满 60 秒就退出报错，`task submit` 连不上当场报。

正文第一行是「task-server：任务 <号> <结局或提醒种类>…」或「task-server：服务健康 <哪一样>」，后面照 `wait` 打的那几行写原因、出路、日志，末尾是 `wait` 那一行原样，只写事实与出路。发不出去（socket 不在、连不上、会话已关）不重试到死：服务日志与告警记录写一条 `inbox-unreachable`，那条任务最后一次尝试的日志接一行 `TASK_SERVER_INBOX` 起头的，任务照旧。收方空着时 Claude Code 照这条消息起新的一轮；收方是跳过权限确认的会话时，消息扣下等用户批准。

`task wait` 留着：照旧取得到结局与提醒，会话不必再为了被叫醒而起它。`task-server/control watch` 已删（退 2、打出路句）；人手查一次服务健康用 `control check`。

行的写法：先发一行 `{"type":"auth","token":"<会话令牌>"}`，再发一行 `{"type":"user","message":{"role":"user","content":"<正文>"}}`；两行备好再连、一次写完（Claude Code 30 秒里收不到一整行就关连接）。来源：收件 socket 与认证行见 Claude Code 文档 https://code.claude.com/docs/en/cross-session-messaging.md「The session's inbox socket」一节；user 那一行的写法文档页没写，取自 Claude Code 二进制自己的调试日志里「[uds-messaging] Inject messages」那一句给的示例。文档页没写行格式，升级 Claude Code 之后要复核。

## 源文件头的声明

服务按源文件头前 60 行里的几行声明认一条命令要什么，调用方照旧不写卡、不写额度。声明由管那份入口或那个包的会话加，服务只读：

| 写在哪 | 写法 | 服务怎么用 |
|---|---|---|
| 入口脚本 | `# task-server-gpu-arguments: <参数>…`（`*` 是任何参数） | 第一个非选项参数是其中一个就要卡、由服务套清场 |
| 入口脚本 | `# task-server-gpu-cards: <张数>` | 要卡时要同一台的这么多张（下面「要几张卡」） |
| `.rs` 源（编出来的二进制的同名源；`cargo test` / `run` / `bench` 用 `-p` 点名的包里的源） | `// task-server-gpu-cards: <张数>` | 同上 |
| 入口脚本 | `# task-server-cargo-family: <cargo 参数>…`（例 `-p singlefs-verification --release --features gpu`） | 它里面起的 cargo 编哪个包、哪一档、哪几个特性：这条命令同键没有记录时，照同一编译族近几次的编译段峰值起步（下面「峰值起步」） |
| 入口脚本 | `# task-server-cargo-family: -p $3`、`# task-server-cargo-family: -p $--crate --release` | 编哪个包由命令参数定的：`$<n>` 取脚本后面的第 n 个词（从 1 数，选项与它的值各占一个），`$<选项名>` 取那个选项的值（`--crate 值` 与 `--crate=值` 都认，`--` 之后的不认）；提交时照这一条命令填上再归族，有一个填不上（没有那个词、值空着或以 `-` 起头）的不归族、照 4G 起。例：`scripts/mutation/prove-red.sh --copy <副本> <crate> <变异名…>` 写 `-p $3`，变异名不同的几条按 `<crate>` 归进同一族 |
| 入口脚本 | `# task-server-shards: <参数>…`、`# task-server-resumable: <参数>…` | 分片、续跑（`task_server_lib.py` 文件头「分片」「续跑」） |

要几张卡：用户定的真卡测法是「你需要测试的是 一张卡1gb 2张卡 三张卡 每张卡都能用来分片gc。没有卡报错。 有卡但是失败 报错加cpu兜底」。声明了张数的，服务照多机配置里卡的先后，挑头一台空卡凑得出这么多张、内存又放得下的，把那台空卡里排在前面的这么多张整组分给它：`GPU_CARDS_BY_PRIORITY` 交这几项用空格接的一串，清场整组清，几张在跑时都算占着。多机配置点名的卡哪台都凑不出这么多张的，结局记 `never-fits`（退 241）；张数写的不是一个正整数的，记 `run-condition-unmet`、说明点名哪一份写了什么。`.rs` 源读哪几份：点了 `--test <名>` 的读 `tests/<名>.rs`，只点 `--lib` 的读 `src/lib.rs`，别的读 `src/lib.rs`、`src/main.rs`、`src/bin/` 与 `tests/` 下的，几份都写了取最大，都没写是一张。例：`crates/singlefs-background-gpu/tests/<目标>.rs` 头几行写 `// task-server-gpu-cards: 3`，`cargo test --release -p singlefs-background-gpu --features tests-on-a-real-gpu --tests` 就分同一台三张。

峰值起步（服务自己库里的尝试记录，`task_server_lib.py` 文件头「资源记录」）：
- 编译族：会编的 cargo 按包、档（`--release`）、特性那几样归族，声明了 `task-server-cargo-family` 的入口照声明归族。同键一次跑完的都没有时，以及撞上限之后的每次重排，从同族（别的命令键的也算）最近 5 次的编译段峰值的最大 × 1.5 起步，比原来的上限大才换（与兄弟片、按近几次最高起跑同一个窗口）。编译段峰值是日志里 cargo 打出「Finished … target(s) in …」那一行之前按轮采到的进程内存（`anon` 加 `shmem`）的最大：本机由 `attempt_runner.py` 采，第二台由 `multi-host-run.sh` 照它每一轮的读数采、打一行 `MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK bytes=<字节>`，两台同一种量；没有那一行的（编不过、不是 cargo）这一栏是空的、不借。整趟的峰值与撞了上限那一次的上限只给同一命令键自己：族里各条编的是同一份，跑测试那一段要的内存各不相同，借整趟的会把一条单测目标照族里跑得最大的那一条记账（任务 9279 一条 `--exact` 单跑峰值 9.47 GiB，借了同族 GPU 对拍整目标的 42.20 GiB）。之后几次都编得小，大的那一次出了窗口就回落；有自己记录的照自己的。
- 同一计划共用峰值：分片的一片第一次尝试，取同一计划里计划那一步与兄弟片已结束的尝试里按结束时刻最近 5 个的峰值（撞了上限的取那一次的上限）× 1.5，只往上抬，不压到 4G 以下。
- 按近几次最高起跑：新提交的头一次与撞上限之后的每次重排，都照这把键服务自己记着的近 5 次里最高的一次（撞过的上限、跑成的峰值）× 1.5 往上抬，重排的那一次另看编译族与兄弟片（同样只看近 5 次）；能升能降：窗口里刚撞上去的那一档马上看得到，更早的出了窗口就不再算，后面几次都跑得小，起点跟着降。没有任何记录的照旧 4G，有真实峰值记录的不兜到 4G。
- 撞了上限重排时，上一次的编译目录不删，重排的那一次还在同一台就接着用（推的：cargo 照指纹复用已编完的 crate，省多少没量过）；换了台的在那一次起的时候删上一次那一份。两台同一段：本机经 `CARGO_TARGET_DIR` 指过去；第二台的尝试带 `--keep-build-directory`，`multi-host-run.sh` 跑完留着 `builds/<标签>/`，重排的那一次带 `--reuse-build-directory <上一次的标签>` 接着用，换台或有了结局时服务经 `multi-host-run.sh --remove-build-directory <标签>` 删。

## 暂停与放开

换 prune-server 的二进制、清判定库旧行这类活要一段别的会话的任务都不在跑的空档，而这类活自己常要经 `task submit` 交命令。`control stop` 做不到：服务停着时谁都交不进任务，正在 `task wait` 的过 60 秒退 78。要空档就用暂停，并放行做活的那个会话（它交的任务暂停着也照常起）：

```bash
python3 task-server/control pause --reason "清判定库旧行" --allow-session <做活的会话>   # 暂停：别的会话的任务不起；做活的会话交的照常排、照常起；提交照收，status / wait / log / cancel 照答
python3 task-server/control check                                                          # 反复看，到打出「…、放行会话 <号>、在跑 N 个、别的会话在跑 0 个」为止
# …做要空档的活（做活的会话经 task submit 交命令）…
python3 task-server/control resume                                                         # 放开：排着的照常起，放行一并清掉
```

- 做活的会话的号：在那个会话里跑 `python3 task-server/task status`，第一行「会话 <号>：…」那个号（就是交任务时服务记下的会话，`status` 里每条任务的会话同一个字段）。放行的是会话，不是某一条命令：那个会话已经排着的、之后交的、分片的子任务与估算子任务、撞上限重排回队的都照常起；别的会话的、没有会话的人手任务照旧排着不起。放行会话排着的任务 `status` 照常写它真在等什么，别的会话排着的写「在等：服务暂停中（原因：…）」。
- 已经暂停着再 `pause --allow-session <另一个会话>` 退 0，换放行的会话（原因与起点不改）；不带 `--allow-session` 的 `pause` 不动放行；`resume` 把放行一并清掉。要一段什么任务都不在跑的空档：不带 `--allow-session` 地 `pause`，等 `control check` 报在跑 0 个。
- 暂停着，排队线程不起任何新尝试（放行的会话交的除外）：插空、分片的子任务与估算子任务、撞上限重排回队的都不起；在跑的照跑到结局（结局照写、该重排的回队，回队的不起）。`pause` 与起尝试共一把锁，`pause` 退 0 时正在起的那一条已经记下，`control check` 的在跑数不会再被挤进一条。
- 暂停态连同放行的会话记在服务的库里（带签名，改库里的放行会话按校验不过算、不放行谁），服务重启后照旧暂停、照旧放行，直到 `resume`；`pause` 已经暂停着的退 0、不改原因与起点。
- `control check` 另打一行「暂停中（原因：…，起于…）、在跑 N 个」，放行了会话的写「…、放行会话 <号>、在跑 N 个、别的会话在跑 M 个」（M 是在跑的任务里不是放行会话交的），仍退 0，也不判「派活判排头能起却没起成」、不看进展；`/health` 带 `paused` 与 `pause`（含 `allow_session`、`other_running`），`ok` 不变；排着的任务 `status` 写「在等：服务暂停中（原因：…）」。
- 暂停着服务照样盯自己的健康（库写不写得进、单元在不在、NFS、第二台），查出毛病照样发消息给名下有任务在排、在跑的会话。
- 只许本机起服务的那个用户发 `pause`、`resume`，认法同提交、取消：服务看调用方的 uid 是不是它自己的。
- 空档的活做完一定要 `resume`；暂停着的服务平时看不出异常（`check` 退 0），忘了放开排着的任务会一直等。

## 自证

`python3 task-server/task_server_lib.py --selftest`：每格一份真的 uvicorn 挂在临时 socket 上，内存包装、第二台、清场、`control` 用的 systemctl 换成假脚本，尝试的 scope 换成假目录，不起编译、不连第二台、不上卡。它跑的全是模拟世界，主 agent 直接起、不经服务排队（钩子名单 `.claude/hooks/lib_project_code_runs.py` 的 `MAIN_AGENT_TOOLS_RUN_DIRECTLY`）；`TASK_SERVER_SELFTEST_CELLS=<格,…>` 只跑点名的格。各格并行的路数取每台默认重活线程数的登记值（`scripts/hosts/heavy_threads.py` 现读），取不出（登记文件不在、写坏，`SINGLEFS_THREAD_CAP` 不是正整数）整份判红、不退回按核数。弄坏开关 `TASK_SERVER_BREAK=<项>` 的表在 `task_server_lib.py` 文件头。
