# task-server

**中文** · [English](README.en.md) · [日本語](README.ja.md)

异常退出后的资源占用、远端清理回执与诊断日志见[任务收尾与 GPU 阶段日志](cleanup-protocol.md)。

singlefs 的统一任务服务：一切跑项目代码的任务（轻、重）都经它起，进同一个队，由服务按资源与公平规则调度、不另设延迟：排头放不下时后面放得下的可以先起（插队），提交满两小时的排到最前，规则见「## 调度」。任务只由主 agent 交（客户端 `task-server/task`，人手同样用它）；子 agent 只用 `status`、`log` 看结局；挑机器、定内存额度都由服务定，提交时不收这几样。服务只调配 CPU 与内存，不分卡、不领卡、不清场：卡归 GPU 批处理验证服务（`gpu-verification-server/`），见「## 卡」。

服务是唯一读写库的进程：FastAPI 跑在 uvicorn 上，作为用户级 systemd 服务 `singlefs-task-server.service`，只开 unix socket（`${XDG_RUNTIME_DIR}/singlefs-task-server/server.sock`，目录 700；全路径至多 107 字节，超了服务、客户端、`control` 都退 78，点名那个路径、它的字节数与上限），不开 TCP。库是内嵌的 SQLite（WAL），放在 `~/.local/state/singlefs/task-server/`，每一行带 HMAC 校验列。内存准入只在服务里判一处：放得下才起，放不下的留在库里排着；起任务仍套 `scripts/hosts/run-with-memory-cap.sh`（本机，设了「已由 task-server 准入」的开关，只做每条上限与 slice 总上限的内核层兜底）或 `scripts/hosts/multi-host-run.sh --peer`（第二台）。

| 文件 | 做什么 |
|---|---|
| `task` | 客户端：`submit [--cwd <目录>] [--manual] [--whole-package-confirmed <确认文件>] -- <命令…>`、`confirm <号> --items <N> --tasks <M>`、`confirm <号> --why '<…>'`、`wait <号>`、`status [<号>] [--json]`、`cancel <号>`、`log <号>`、`clean <号>`、`promote <号>`（把这个会话在排的一条提前到队头：下一轮排最前、不过 CPU 闸、内存放得下就起；只许提交它的会话）；用法与退出码在文件头 |
| `control` | 装与查：`install [--dry-run]`、`check`（服务的 `/health` 只报四样：库写不进；派活判排头能起却连着三轮没起成，判不能起的——等负载、排在排满时限的任务后面这类——不报；看进展：某台照公式放得下排着的最小那条、却 10 分钟一条都没起，兜派活自己判错；本机读不到在跑尝试的单元状态，`unit_unreadable` 栏逐次列出，见「## 读不到单元状态」；多机配置开了 GPU（`ENABLE_GPU=1`）的接着起 `gpu-verification-server/control check`，它退非 0 时退 78、出路写起它的命令，没开的不查）、`start`（开了 GPU 的在任务服务起好之后接着起 `gpu-verification-server/control start`，没开的不起）、`stop`、`clear-queue`（服务停着时清掉库里的任务与尝试两张表，先备份库，资源记录不动；库里还有在跑的尝试——停服务不停它们——就拒、库不动）、`pin-cap <命令…>` / `unpin-cap <命令…>`（经服务给一条命令钉上或撤掉内存上限的钉值，回读过才退 0）、`pause [--reason <一句>] [--allow-session <会话号>]` / `resume`（暂停与放开排队：暂停着照收提交、不起新任务，只有放行的会话交的照起，见「## 暂停与放开」） |
| `task_server.py` | 服务（`serve`）：接口、排队、准入、起单元、对账、重试、分片、资源记录；判法在文件头 |
| `attempt_runner.py` | 每一次尝试的单元里起命令、记退出码与资源（内存峰值取按轮采到的与内存包装退出时读的里大的）；编译目录任务有了结局由服务删 |
| `task_server_lib.py` | 共用的那一份：路径、依赖、错误类型与退出码表、重试、资源记录、编译目录、分片、环境变量名单、身份；整套的自证 `--selftest` 与弄坏开关表在它文件头 |
| `task_server_selftest.py` | 自证的各格（由上一行起） |
| `singlefs-task-server.service.in` | 用户级单元模板 |

没有自己的配置文件：调度资源照仓根的多机配置 `multi-host.env`（模板 `multi-host.env.example`）读；socket、库、日志、签名密钥、编译目录用 `task_server_lib.py` 文件头「路径」写的固定缺省值。必须依赖：fastapi、uvicorn、`multi-host.env`（含 `NFS_SHARED_ROOTS`、`SHARED_DATA_ROOT` 与 `HOSTS`，写法见「## NFS 共享」的「多机配置写哪几键」）、NFS 共享配好（见「## NFS 共享」）、服务通。

## 调度

挑机器按余量均衡：任务在派得过去的几台里挑放得下的，放得下的不止一台时挑放下之后剩余占那台总上限比例大的，一样大取本机；撞上限重排不钉台。分片的各片照计划先去第二台，放得下就不按均衡改台；计划、各片与合并都不钉台，计划里写了 `pin` 的整件记 `shard-plan-refused`（片不钉台，哪台由服务挑）。第二台在同一棵共享的树里跑：仓、工作根 `~/.local/share/singlefs/work/` 所在的共享数据根与会话记录目录经 NFS 两台挂在同一绝对路径（多机配置 `NFS_SHARED_ROOTS`，下面「NFS 共享」一节），派第二台的那一次经 `scripts/hosts/multi-host-run.sh --peer` 在任务的工作目录、同一个绝对路径里直接起，不拷树、不送、不取回、不改写路径，写出的文件两台都读得到；编译目录由 `multi-host-run.sh` 在第二台本地盘上这一趟新建（多机配置 `BUILD_ROOT_BY_HOST` 里第二台那一项下的 `builds/<标签>/target`，不在任何共享根下）、命令结束或被停都先删掉（会编的命令撞上限重排时留着给下一次，见下面「峰值起步」），服务不设、只记下它在哪（`task status` 看得见），`task clean` 不管它。派不派第二台只看第二台此刻用不用得了与余量；派不过去的三种在提交时就拒（下面「## 提交写法」）。没有只派一台的名单（用户：「【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：点到哪个目录、哪个入口的命令都派得到两台，哪台由余量定；派到第二台跑红的照红，由脚本那一侧修，服务不替它挡。合入后验证 `scripts/gate/merge-and-verify.sh` 只在自己拷的快照上打补丁与验、主工作区一个字节不写（它文件头「快照（不拿锁）」），不算改主工作区，撞上限照常放大重排。改主工作区的工具（`hook-swap.py`、`apply-writer-patch.py`、`stage-mine.py`、`scripts/kb/` 下的）与带 `--write` 的派得到第二台（改的就是同一棵树），撞上限这类本会重排的结局照旧不重排、打回提交它的会话；所以括号里那几样工具首趟就给足：上限不低于「（同键近 5 次最高 + `hook-swap.py` 的 `--verify` 给的验证命令同键近 5 次最高）× 1.5」（不设 4G 底；任务 8367 照同键窗口只给 857M，撞了、换上做了一半）。分片的计划目录与各片的产物目录在共享工作根下的 `task-server-shards/`，两台照同一个绝对路径读写；计划那一步往计划目录下放的测试二进制（`test-binaries/`）在父任务有了结局（成、败、取消都算）与 `task clean <父任务号>` 时由服务删，计划目录别的文件留着，那个子目录的真路径不在这一条任务自己的计划目录下（指到别处的符号链接这类）的不删、服务日志打一行，删了几份、多大写进操作日志。服务不认合入锁：主工作树正在合入时照常排、照常起。一轮至多在第二台起 4 条（一齐起太多 ssh 会被重置），起满了 2 秒后再排一轮；服务自己起的 ssh（NFS 复核、余量探测、核进程、进展读数）一律复用连接：带 `ControlMaster=auto`、`ControlPath=<服务状态目录>/ssh/%C`（每台一份）、`ControlPersist=300`，主机名只从多机配置取，停服务时逐个关掉主连接（尝试经 `multi-host-run.sh` 起的 ssh 不在这一层）；第二台的结局照 `multi-host-run.sh` 打进日志的那条命令的结束行（`MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>`）定：有这一行按命令自己的退出码定，命令自己退 255 记 `command-failed`、不重排；没有这一行而退 255 才是第二台没跑成（连不上、ssh 被重置），记 `peer-run-failed`、原样重排，至多 2 次。排队先后只照两条规则：① 排满时限（两小时）还起不来的任务挡住排在它后面的全部任务，它起来之前后面的都不起，排队原因写「排在任务 N 后面：它已排满时限」；② 前面有没起的任务时，后面的只许这样的先起：它有上次的运行时长（同一命令键上次真跑完用了多久，是记录、不是估的），而且这个时长不超过那台在跑的任务剩余时长里最长的那个（剩余 = 那条任务上次的运行时长减已跑；在跑的都算不出剩余时长时，有上次运行时长的许先起），没有上次运行时长的不插队，排队原因写「不插队」。服务不预计用时：内存照记录定上限、撞了乘 1.5；一项实际跑满四小时停掉打回，跑满两小时提醒拆分（带 `--full-gate` 的提交门禁不受这两条限）。排着的一律不留账。两台走同一段判法（记账、放不放得下、卡住与 CPU、峰值、上限、编译目录、挑台、盘余量，盘余量两台同一个 50G 下限），按台不同的只在怎么连上、怎么量、怎么起那一台（`task_server.py` 的 `Machine`）。尝试正常结束之后单元里还留着命令起的孙进程的，服务停掉、结局说明写明；跑超过 30 分钟而失败、脚本又没在文件头写 `# task-server-resumable: <词>…` 声明续跑的，结局说明写「这条跑了 N 分钟、没有续跑：重交会从头跑」。

每次尝试的命令外面套 `scripts/hosts/capped.sh <每台默认重活线程数>`，线程一律是这个数，提交时不收；外面不套清场包装（任务不拿卡，见「## 卡」）。内存包装、`capped.sh` 与 `multi-host-run.sh --peer` 只认服务直接套的那一层：服务给每一层设一个准入标记（值是 `<任务号>.<第几次>`），那一层读掉、不交给命令，任务里面的脚本再套一层当场拒，出路是删掉那一层，内存、线程照读服务交的 `TASK_SERVER_MEMORY_CAP_BYTES`、`SINGLEFS_THREAD_CAP`；服务自己环境里的选卡变量（`GPU_CARDS_BY_PRIORITY`、`CUDA_VISIBLE_DEVICES`）不交给命令；会编的 cargo 另给这一次一个仓外的 `CARGO_TARGET_DIR`，有了结局服务就删（提交时设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1` 的留着，`wait` 末行带 `build=<目录>`，由提交它的会话 `task clean <号>` 删）；取消的在同一次取消里连现场一起删（下面「## 提交写法」的取消）。定额度只照服务自己库里的尝试记录（近几次跑成的峰值的最大值 × 1.5；不读内存包装的峰值表；命令键先剥掉 `env` 与多机配置路径、`LANG` 这类不改跑法的变量前缀），没有记录的先记 4G（只判红过的不低于判红那一次的峰值 × 1.5），有记录的照记录 × 1.5 记上限与记账、不抬到 4G（钉值照旧），撞了乘 1.5 重排（重排的也不兜到 4G），乘 1.5 会到这台总上限的不再重排，有总上限更大的另一台就换过去，没有就记 `memory-cap-at-maximum` 打回提交它的会话，`next=` 与发给会话的正文写撞了哪台、到了多少、峰值多少，修改意见是核实这一趟的范围（点的项、测试目标）、分片与同时起的个数，收小或分片再交；定额度只读峰值口径一栏写着「进程内存」的尝试记录，没有这一栏的旧记录（含页缓存）不读；算这台还放得下多少只算进程用的内存（`anon` 加 `shmem`），页缓存与内核挂在 slice 上的不算；起来之后先宽限 10 分钟不判；之后日志满 10 分钟没涨、这段时间 CPU 也几乎没涨（平均不到一个核的 1%）的服务停掉、记 `stalled`，读不到 CPU 不当没涨；日志与 CPU 都满 10 分钟不涨、而这段时间每一轮都有进程在 D 状态（不可中断的 IO 等待：本机读这一次尝试各进程 `/proc/<号>/stat` 的状态，第二台读 `multi-host-run.sh` 每轮打的 `MULTI_HOST_RUN_PEER_IO_WAIT poll=<轮> blocked=<D 状态的进程数> read=<yes|no>`，这一行不算日志在涨）的是卡在 IO 上，不记 `stalled`、不停，有一轮没有 D 状态的进程或读不到的照原判法判（任务 8608、8609、8611、8619 在第二台上等本机导出的 NFS 时被当成卡住停掉）；跑过记录用时 5 倍、日志又满 10 分钟没涨的不看 CPU 也停掉、记 `stalled`（死循环也停），没有记录用时的不判这一条；记录用时只认跑过的活与这一次相当、又不少于 3 次跑成的（分片的片只认片数相同的那几趟），不够的不判这一条；进程内存或峰值一个窗口里涨了至少 64 MiB 的，算有进展，跑过 5 倍也不停；命令自己按固定间隔打的心跳行（`TASK_SERVER_HEARTBEAT <说明>`）不算日志在涨，只剩心跳行、CPU 也不涨的照样满窗口判卡住，夹着真输出的不停；命令被信号终止的（尝试单元收到停的信号，或退出码是 128 + 信号号），结局的理由行写出信号名，并写服务这一次有没有发过停的信号、是哪一步发的；`stalled` 不重试，打回提交它的会话；判卡住那一刻这一次进程用的内存（`anon` 加 `shmem`，同上面算余量的口径；页缓存撑满上限时内核回收它、不杀进程，不算）此刻或峰值已到上限 98% 的是顶在上限上卡住，照样停掉，但记 `memory-cap-hit`、乘 1.5 重排（钉值的打回 `memory-cap-at-maximum`），说明写「顶在上限上卡住」；已跑多久、记录用时、在用核数只写进 `task status`。记的内存峰值两台都只算进程用的内存：每一轮采样读内存包装那一层 scope 的 `memory.stat` 的 `anon` 加 `shmem`、留见过的最大（任务 8853 编验证包时进程不到 1G，页缓存把 scope 的 `memory.peak` 撑到上限 50176M）；比一轮还短、一次都没采到的才照包装退出时读的 `memory.peak`。第二台那一层的内存峰值照 `multi-host-run.sh` 打的 `MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<字节>` 记进那一次尝试，同本机一样进定额度，不打的记没量到。编译目录占盘也按命令记峰值（本机量服务给的编译目录，第二台照 `MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<字节>`），派到一台之前那台的盘余量减去在跑的还要涨的，要放得下这条命令的占盘峰值加 50G 下限，放不下就排着、排队原因点名「编译目录占盘」。命令自己退 75（prune-server 不可用）记 `prune-server-unavailable`、`wait` 退 75、不重排，服务另起一次 `python3 prune-server/control check`，不通写告警，不起、不停 prune-server。命令写成 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>` 的，算命令键、认登记表行照 `--` 后面的原命令（`--server-binary` 的路径不进键），命令键前面另接「〈task-server 临时 prune-server〉」：同一个 scope 里多起一份 prune-server，不与不包的那一条共用记录；包装照旧整条执行。

准入记账：新起的按上限记（收到那台总上限减那台此刻的余量为止）；在跑的只照报告里它的 scope 此刻回收不掉的用量记（不加固定余量、不看任何预先估的数）、不超过它的上限（报告里还没有它的 scope 的照整份上限记），各自的上限（memory.max）照旧，超了只杀它自己；每台的公式是总上限减余量减在跑的记账之和（记账之和不超过总上限；在跑的上限之和不进公式）；每条的上限只照这把命令键自己的记录定，没有记录的从 4096M 起、撞了乘 1.5 重来，别的键与同一趟的兄弟片都不借；slice 兜底：那台 slice 实测还放得下的不到 4 GiB 时这一轮不派新任务，排队原因写明是实测余量不够；内存放不下的排队原因写明两个数各是什么——账面还能记多少（总上限、余量与在跑的记账之和）、slice 实测还放得下多少——并点名这台记账最多的两条在跑任务与各自的记账、实占；CPU 闸照负载与在跑尝试的线程之和取大，加上这一条的线程，不超过核数 × 3.4375（32 核的台线是 110）才派；在跑的每条起跑满 120 秒、服务量满了一段的照它最近量满的那一段的在用核数记：服务每一轮读一回累计 CPU，两端相隔满 120 秒的算一段的在用核数，取最近那一段（向上取整、至少 1、不超过登记数），任务忙完一段闲下来账上跟着降；各段取最大的峰值段核数只给收尾记的核数记录（`peak_segment_cores`、`cpu_gate_threads`）与要核数少的名额用；机器挤满时在用核数被压低（几条各要 31 核的挤在一台上各只拿到七到十核），所以服务每一轮另读一回这一次尝试的可跑线程数（本机由每一次尝试那一层采样时数进程树里状态 R 的线程，第二台由 `multi-host-run.sh` 每轮打一行 `MULTI_HOST_RUN_PEER_THREADS`），在跑的那一支记 min(登记数, max(最近那一段的在用核数, 最近一回读到的可跑线程数))，读不到的那一轮只照在用核数。起跑已满 120 秒而服务还没量满一段的（刚重启）照到此刻的累计 CPU 除以已跑秒数记（累计是 cgroup 记的、重启不丢）。起跑没满 120 秒的、量不到的与新起的这一条取法相同：照同一命令键近 5 次跑到头的 CPU 闸线程数（尝试收尾时记的 `cpu_gate_threads` = min(登记数, max(峰值段核数, 可跑线程数的最大))；没有这一栏的旧尝试不读）取最大（向上取整、至少 1、不超过 31）算，没有记录或没有命令键的先按登记的线程数算（量满一段之后照实测）；超线的这一条要的核数不超过 2 时照样放行（要核数少的名额），每台同时靠名额在跑的不超过 4 条，满了照排队、原因写「要核数少的名额 4 个已用满」，靠名额起的尝试记 `cpu_small_slot`、起跑那一行与 `task status` 写明，名额照库里在跑的尝试数，服务重启后照样认，起满 120 秒、峰值段核数超过 2 的不再占名额（量不到、还没量满一段的照占着）；读不到核数或负载的那一台这一轮不派；第二台的余量由服务的一个后台线程定时读（带时限，读不成的一段时间里不再去连），派活只取它最近一份，第二台卡住不拖住本机派活；两台同一段：用量由读那一台余量的那一次（第二台是同一次 ssh）按每条尝试的 scope 量回来（scope 照里面进程环境的 `TASK_SERVER_ATTEMPT_LABEL` 对到尝试上，两台都给），报告里没有那一条的 scope 记整份上限、不记 0，第二台那一层内存包装不另记账。余量是那台此刻 `singlefs-heavy.slice` 里回收不掉、又不归任何在跑任务的那一截（slice 自己记着、哪个子 cgroup 里都没有的，量法在 `scripts/hosts/memory_peaks_lib.py` 文件头「余量」），每次读两台的余量时各量各的，按 1 GiB 向上取整（量到 1.01G、1.1G 记 2G，2.1G 记 3G，0.7G 记 1G，恰好整数的不进位）；它不是常量，也不在配置里。哪一台量不到余量，那一台这一轮不派，排队原因写哪一台与原因；能派的每一台都量不到的不起、记 `run-condition-unmet`（`wait` 退 78），不退回固定的数。每次尝试经环境 `TASK_SERVER_MEMORY_CAP_BYTES` 把这一次的上限（字节，十进制）交给命令，本机与第二台都给。`python3 task-server/control pin-cap <命令…>` 给一条命令钉上限（`unpin-cap <命令…>` 撤；经服务写库，服务在跑时也能用）：钉了的命令上限与记账都取多机配置 `SLICE_MEMORY_CAP_BY_HOST` 里派到的那一台那一项减那台此刻的余量（`<地址>=<值>` 空白分隔，写的是那台 slice 的总上限，配多少就是多少；每次现读，不记字面的数；那一项不比那台的余量大的，`pin-cap` 当场拒、出路是把那一项改大），学到的峰值不顶掉它，撞了不放大、记 `memory-cap-at-maximum`，`task status` 写明是钉值；钉了而配置的 `SLICE_MEMORY_CAP_BY_HOST` 读不到（没有、写法不对、少一台、多了不在 `HOSTS` 里的地址、文件读不了）的不起、记 `run-condition-unmet`（`wait` 退 78），`wait` 的理由行写读不到的原因与出路（改对配置或 `unpin-cap`）。疑似卡住只通知：宽限过后整个单元的 CPU 满 10 分钟平均不到一个核的 1% 的，不管日志涨不涨，服务不停它：`task status` 写「疑似卡住」与依据、记一条告警，`wait` 打一行 `TASK_SERVER_SUSPECT task=<号> …`、退 231，任务照跑，再起 `wait` 接着等（同一段只报一次，CPU 又涨了算这一段结束）。日志满 10 分钟没涨、此刻内存不低于这一次上限的 98% 的，CPU 在涨也停掉（在回收里打转不算在算；任务 7723 贴着 6144M 的上限这样跑了 37775 秒）、记 `memory-cap-hit`、乘 1.5 重排；两台同一段判，本机读外层 scope 的 `memory.current`，第二台读 `multi-host-run.sh` 每轮打的 `MULTI_HOST_RUN_PEER_MEMORY` 一行（这一行与 `MULTI_HOST_RUN_PEER_CPU` 一样不算日志在涨）。点到 `.claude/gate.d/` 的门禁派得到第二台（它跑的就是这棵共享的树）；带判定库而登记表 `.claude/gate.d/verdict-library-scripts.tsv` 那一行第三列还没写 prune-server 的，提交时就拒（下面「## 提交写法」）。`/health` 只做库写入试探与排队线程心跳，排头放得下却不起由排队线程每一轮算好、`/health` 只读。

跑满两小时提醒：提交时的门禁之外，一条任务各次尝试在跑的时长累加（排队等的不算；撞了内存上限、照大一档重来的计时归零，只从最后一次撞上限之后那一次起算，撞几次都一样；分片的各片各自累计）满 120 分钟，服务不停它、不改任务，记要处置的提醒 `long-run-unconfirmed`，推给会话的正文写命令、范围声明、累计与各次用时、日志末几行，请它核两件事：范围对不对（点名范围：`--item`、`--check` 少几格、点名测试目标；给补丁目录；或照服务分片；整轮的留到提交时）、这条是不是确实要跑这么久。范围不对的 `task-server/task cancel <号>`、改了范围重交；确是要跑这么久、范围对的，照推送里那一整行 `task-server/task confirm <号> --why '<为什么要跑这么久、范围为什么是这些>'` 确认：`--why` 去掉空白至少 12 个字，只许提交它的会话，只对在跑、已满两小时的任务（不合的退 3）；确认了记「长跑已确认」，`task status`、列表、推送与日志头打一行「长跑已确认：<why>」，之后不再提醒，别的判法照旧。没确认的满 30 分钟再推一回、之后每翻一倍再推，任务一直跑。没有会话的（人手 `--manual`）不推，写进任务日志与 `status` 的「长跑待确认」那一行，同样可以确认。不提醒的只有带 `--full-gate` 的提交时门禁：提交时的六道门禁（`crash-point-replay`、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation）、崩溃放量全部分项（`scripts/crash/crash-amplification.sh`）或 `scripts/gate/gate-staged.sh`，不带 `--check` 与 `--item`；带 `--check` 的门禁、`merge-and-verify`、入口脚本、实验都提醒。

四小时硬上限（异常表 `over-four-hours`，动作打回，退 229）：带 `--full-gate` 的之外一律判，`--why` 与「长跑已确认」都放不过。累计的算法同上一段（排队等的不算；撞了内存上限重来的计时归零），满 14400 秒服务停这一次尝试、不重排，记结局 `over-four-hours`，推给提交它的会话；服务重启时已在跑的同样判。出路是照 `.claude/rules/long-run-split.md`「怎么做」把跑得太久的那一项拆开（缩域、少点几项不算拆），再用 `task-server/task submit` 重交。分片的：计划、各片与合并的子任务各按自己的尝试累计，整件不另算挂钟（排队把片一片片串起来的时间不算进谁的账）；一片满界只停那一片、记这个结局，兄弟片照排、照跑，都收齐了整件照那一片的结局收尾、不合并。改主工作区的命令同样停，同时推 `workspace-write-interrupted` 请它核现场。

人手交任务：不在 Claude Code 会话里交的，写成 `task-server/task submit --manual -- <命令…>`。不读收件地址，任务记无会话，服务不推结局与提醒，结局用 `task-server/task wait <号>` 取；确认那一步照旧。会话是服务照进程树读的：会话里带 `--manual` 的拒（`manual-from-session`），没有会话又没带它的也拒（`sessionless-unmarked`），环境里填收件地址不算数。

整包测试要确认标识：`cargo test` 不点名 `--test`、`--bin`、带测试名的 `--lib`，或带 `--lib --tests`、`--workspace`；`.claude/gate.d/crash-point-replay.sh` 的 `--check` 点了 feature-gated-tests、product-gpu-real-card-tests，或点了 crash-case-replay 而不带 `--crash-cases-only`（点哪一格、点几格都算；`--check crash-case-replay --crash-cases-only --item crash-case:<名>` 只判点名的崩溃枚举用例，不算）；不给补丁目录或带 `--all-packages` 的 `scripts/gate/merge-and-verify.sh`——这几种是整包测试。不指定范围的跑法同样要确认标识（提交时的门禁另有 `--full-gate` 一条，见下）：提交时的六道门禁与 `gate-staged.sh` 不带 `--check`、`--item`，`scripts/crash/crash-point-replay.sh`、`scripts/crash/crash-amplification.sh` 不带 `--item`，`scripts/experiment-apparatus/replay.sh` 不给实验号（表在 `task_server_lib.py` 的 `UNSCOPED_RUN_ENTRIES`）；不指定范围的只在提交时跑，提交之外要跑先问用户。两种没带 `task submit --whole-package-confirmed <确认文件>` 的提交时当场拒（退 3，异常表 `whole-package-unconfirmed`）；提交时的门禁（六道、崩溃放量全部分项、`gate-staged.sh`，不带 `--check`、`--item`）不走整包确认，带 `--full-gate` 交，没带的当场拒（退 3），整包确认标识顶替不了。提交时拒的全部在服务的 `submit_refusal` 一处判，服务起来时重核在排与待确认的任务照同一份判，核不过的记 `resubmit-required`、推给提交它的会话。确认文件照 `.claude/agent-common.md`「日常开发的测试不跑整个包」写三遍确认（①改到的文件与函数、②引到它们的测试目标与函数及 grep 命中、③为什么只点名②不够），放共享根下、写绝对路径；服务只判形式（读得到、三段都在、每段去掉空白至少 12 个字），不合的退 3、点名缺哪一段。核过照收，`task status`、列表、推送与日志头打一行「整包已确认：<文件>」；带它照样受两小时提醒。

启动测试与验证要点名（异常表 `unnamed-run`，提交时拒，退 3）：提交的命令与 `bash -c` 里命令位置上的每一条，`cargo test` 要在 `--` 之后带 `--exact` 与测试全名（`cargo test -p <包> --test <目标> -- <测试全名> --exact`，只点 `--test` 目标、只给筛选词的都不算）；`scripts/gate/merge-and-verify.sh`、`scripts/crash/crash-point-replay.sh`、`scripts/crash/crash-amplification.sh` 要带 `--item`；`scripts/mutation/prove-red.sh` 要在 `--copy <副本> <crate>` 之后写变异名；`.claude/gate.d/` 的门禁阶段要带 `--item`，或只用 `--check` 点不起测试的格（起测试的格手列在 `task_server_lib.py` 的 `TEST_STARTING_GATE_CELLS`，点了其中一格的要带 `--item`；各道文件头的格名表不标哪一格起测试，读不出来，自证 test-starting-cells-listed 核表里每个格名都在那一道的格名表里）；门禁 `crash-point-replay.sh` 带了 `--item` 还要 `--check` 把起整个验证包的格排掉：点崩溃枚举用例的写 `--check crash-case-replay --crash-cases-only --item crash-case:<用例名>`，不带 `--check` 的照旧九格、拒；只点 `feature-gated:<文件名>` 而不带 `--check` 的只跑 feature-gated-tests 一格、只编点名的文件，照收。只查不跑的写法（`--list`、`--list-items`、`--selftest`、`--no-run`、`-- --list`）不判；带 `--full-gate`（提交时那一趟）或 `--whole-package-confirmed`（用户确认过的整包）的不判。拒绝说明写哪一种没点名，并照这条命令写出改成点名的样子。判据只一份（`task_server_lib.py` 的 `unnamed_run_problem`），范围声明闸 `.claude/hooks/run-scope-guard.sh` 在命令执行前调同一个函数判经 `task submit` 交的那一条；服务起来时重核在排的照同一份判。

内存最高那一刻的明细（任务 10279 峰值 45.46 GiB，包装退出时读 memory.stat 只剩 anon 311296：进程已被杀、内存已释放）：跑的过程中每轮读 scope 的 memory.current，比见过的最高还高就存一份 memory.stat 的 anon、file、file_dirty、file_writeback、shmem（本机 `attempt_runner.py`，第二台 `multi-host-run.sh` 每轮打一行 `MULTI_HOST_RUN_PEER_MEMORY_STAT`），撞上限那一次服务把它换成一句抄进结局说明与推给会话的正文，写明是内存最高那一刻、大头是进程自己的内存（anon）还是页缓存（file，含没写回的脏页）；只记，不改判法。峰值两台同一个口径：按轮采的进程内存与内存包装退出时交的（扣页缓存、每 0.1 秒采一次）取大的，第二台读不到后者的记没量到、不进定额度（任务 10045 这一族在第二台每次只读到外壳的 18 MiB，本机量到 556 MiB）。连着撞上限之后打回的（`memory-cap-at-maximum`，或撞满 16 次的 `memory-cap-hit`），结局所在那几次尝试末尾连着撞上限不少于两次的，`wait` 的理由行另带 `loop_leak=`、推给会话的结局正文另起一行写「检查代码有没有逻辑循环或内存泄漏：…」：要查的两类（逻辑循环：死循环、无界递归、反复往同一集合里加、每轮都重读全量；内存泄漏：只增不减的缓存或表、把大文件或全量结果整份读进内存、用完不释放的大对象），依据是各次的上限与峰值逐次列出、峰值是不是一次比一次大、每次是不是都顶到上限的 98%、最后一次撞上限那一刻的内存明细（大头是堆还是页缓存）；只撞过一次的、排队那一刻记账就超上限一次都没起的不带。

取消整件与删编译目录的先后：取消分片的整件，在跑的子任务停了单元之后，服务等每一条在库里有了结局（同不分片的取消那一种等法），都到了才给整件记 `cancelled`、删产物、回停了，等不到的回 500、点名哪几条子任务；记下结局时先删完编译目录再叫醒 `wait`；`task clean` 与记下结局时删、取消之后删产物拿同一把锁、锁里现读各次尝试，先到的删、后到的读到已删就不再删；删完记「已删」只改那一栏，库里已结束的尝试不会被写回成在跑。起之前复核现读那一台时照排队那一轮同样判盘余量下限、编译目录占盘与第二台的 NFS 挂载，复核的结果同时换进这一轮的余量。判卡住停的只是判它的那一次尝试，这一会儿重排起的下一次不受牵连；同一次尝试的第二台清理恢复（`multi-host-run.sh --recover-cleanup`）同一时刻只起一趟。

排队原因逐台写：排着的任务在等什么（`task status` 的「在等：…」，排队过久提醒的依据也是它）每一台各写一句、句首是台名，句里点名挡着这台的是哪一样：内存、编译目录占盘、CPU 闸（连同这台读不到核数或负载）、slice 兜底（这台 slice 实测余量不到 4 GiB）、钉值，或这台此刻用不了；只写数、要读的人自己推是哪一样挡着的不算写明。整条打回的（`never-fits`、`run-condition-unmet`）说明同样逐台带台名。

处置的依据从库里读：提醒、确认、照跑、打回这类处置要用的依据（报过哪一种、报到哪道提醒门槛、确认过没有、记没记照跑）写进服务的库，判的时候照库里那一份认；服务进程内存里只留这一轮算的、服务换过之后照库与现读的数重算得出来的东西。服务停了再起，在排与在跑的任务照库接着认：报过的同一道门槛不重报，确认过与记过照跑的不再推，`confirm --why` 对报过的提醒照样收。调度自己的判断（任务外空转进程报过几回，分片推不动的回数）同样落库：库 meta 表里一样一行 `restart-surviving:<名>`，带签名，服务起来读回，签名对不上的不认、记 `record-tampered`、照空的起。

**记账与判法的几处细化。**

- 在跑的内存记账：起跑不满 2 分钟的按整份上限记（还没量到它会涨到多少），满了之后照这一轮读到的 scope 实占记（不超过上限）：编译冲高之后回落，账就跟着降。
- 记账对账：服务每分钟逐台比「在跑尝试的内存记账之和」与对账的基准（各 scope 回收不掉的用量之和），记账之和不少于基准的 3 倍、多出 8 GiB 以上、连着 3 轮，判「记账漂移」（多出来的只来自报告里没有 scope、记整份上限的那几条），逐条列各尝试记多少、实占多少、取的是此刻实测、实测到了上限还是没有 scope 记整份上限；写进 `/health` 的 `booking_drift` 栏与 `control check` 的输出，推给被它挡住、还在排的任务所在的会话（同一件事当场推一回，之后时长每翻一倍再推一回）。只报不停任务。
- 提交进出日志：服务每收一条提交，回话时在服务日志里打一行「提交进出」：收到的时刻、收下还是拒了，与进线程、提交时拒、认会话、认任务、记库、提交时警告、确认单、回话各做完时距收到多少秒；一步本身用了 5 秒以上的当场另打一行。`task submit` 等回话满 30 秒退 78 时，照这几行看慢在哪一步。
- 单元已停先放记账：回读到单元已停（`is-active` 的输出不在跑、`cgroup.procs` 为空；读不到状态不算停）那一刻起，这一次尝试不再进 CPU 与内存的在跑记账，收尾（删目录、取峰值、落结局）照旧做完。
- 排队原因里的 CPU 负载写成「负载 N（其中等 IO 的线程 M）」，闸的判法不变。
- 提醒 `fills-any-cap` 不数上限照记录定的那几次（尝试的 `prediction` 一栏是 `record`）。
- 整机内存线：一台整机可用内存占比与 swap 空闲占比都低于 12%（earlyoom 在两样都低于 10% 时才杀进程）时，这一轮不往那一台派新任务，排队原因写两个比例；只挡新派，不停在跑的。
- 被不是服务发的 TERM 或 KILL 杀掉（earlyoom、内核 OOM）：命令退非 0、服务这一次没有发过停的信号、这一台同一时段的 earlyoom 或内核日志里点名了这一次尝试进程树里的进程号，记 `killed-externally`（退 228），同一上限重排至多 2 次；第二台的进程号服务这一侧拿不到，由 `scripts/hosts/multi-host-run.sh` 在第二台那一侧记下这一趟进程树的进程号，命令退非 0 之后读第二台的 earlyoom 与内核日志，点名了其中一个的打一行 `MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=<earlyoom|kernel> pid=<号> line=<日志原行>`，服务读到这一行照本机那一路记 `killed-externally`、同一上限重排；没有这一行的仍记 `command-failed`。内核那一行是「Memory cgroup out of memory」的（某个内存组到了它自己的上限，比如命令自己造的小内存组让进程撞上限，不是整机内存紧张）两台都不算外部：第二台那一侧 `multi-host-run.sh` 遇到这种行不打 `MULTI_HOST_RUN_PEER_EXTERNAL_KILL`（跳过它接着找下一行），服务那一侧另按 `line=` 的内容分，照原来的路判（命令自己退非 0，或这一条自己的 scope 撞了上限的照撞上限）。
- `python3 task-server/control bookings`：只读，逐台逐条列在跑尝试的上限、此刻实占、记多少与为什么记这么多，末行是这一台的合计与照公式（总额 × 1.5 减余量减记账合计）还放得下多少、实测 slice 还放得下多少（读 `/health` 的 `booking_ledger` 栏，不连第二台）。
- `python3 task-server/control key-stats [--since <时刻>] [--dump <文件>]`：只读（库以只读方式打开，服务在跑时也能用）。统计只取收尾了的、有命令键的、峰值口径是这一版的尝试（与定额度读的同一个判法），首行写筛前筛后的行数；交替与「撞过上限、最近一个任务跑成」按任务数（同一任务的几次尝试折成一条，取第一次尝试的结局），按尝试数的交替另报一行。`--since` 只取这一刻之后起跑的尝试（epoch 秒，或 `YYYY-MM-DD` 取那一天开头，日期照 `time.gmtime` 算）。`--dump <文件>` 把收尾了的尝试逐条写成 jsonl（列 `id`、`task_id`、`number`、`command_key`、`command`、`started_at`、`ended_at`、`outcome`、`exit_code`、`memory_cap`、`memory_peak`、`peak_metric`、`stalled_at_cap`、`prediction`、`shard_count`、`host`、`item_plan`，没有的写 null），文件排他新建，已在的拒、退 2。
- CPU 闸里新任务照历史核数记时，命令自己退非 0、跑到头的几次也算（被停的、撞上限的、被信号杀的不算）。
- `task-server/task promote <号>`（客户端那一条执行前由钩子弹窗要用户确认）：只许提交它的会话，任务要在排或是分片确认过的父任务；提前过的排最前、不算排满时限、起不来不挡后面（后面的不照插队判）、不过 CPU 闸（内存放得下就起）；分片的父任务连同此刻在排的子任务一起提前，之后建的计划、各片、合并照样提前；改完当场排一轮。
- 暂停着放行一个会话时，排着的任务的排位照全队算，别的会话排着的排位不丢。
- 第二台盘余量不到下限时，记录比本机大、只有第二台装得下的任务排着等它的盘腾出来，不判 `never-fits`。
- 库的写事务出了错（盘满、IO 错、忙）先回滚再报，连接不留在写事务里，别的线程照样写得进。
- 第二台上一回读不成、还在冷却里的那一段，收尾、核进程、删编译目录与空转那几处不去连，当场照「连不上」办；停服务时连同第二台的余量探测线程一起等它退。
- `task clean` 同样删第二台留着、记结局时没删成的编译目录（照第二台的删法）。
- 命令起不来（argv 头一个不在、没有执行权限）记命令自己退 127 或 126，不记 `launcher-lost`；命令退了之后服务那一层再采一次 CPU 与线程。

## NFS 共享

**为什么要。** 原则：「所有的脚本必须多机跑。抛出的结果需要多机都能读取。如果不行，要么复制资源，要么共享文件。」NFS 是共享文件这一条路：本机导出若干目录，第二台挂到同一个绝对路径上，一台写出的结果另一台直接读到，任务引用的路径在两台上指同一份东西。NFS 的配置写在这份 README 里，也是服务起来时的检测项。所以服务不替用户配 NFS，配置只有多机配置 `multi-host.env` 里的一个键，没配好服务就不起。

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
- 挂载：第二台上每段 `stat -f -c %T -- <根>/.` 读到 `nfs`（走进去触发自动挂载）；连第二台的 ssh 一次超时先当「慢」，进两分钟的复查期、每隔 5 秒复查一回，哪一回通了只在服务日志与 `/health` 的 `nfs` 栏（`slow_last`）记「慢」、不推提醒、照派第二台，复查期满还不通才算不过（「连不上第二台」，才发消息）；
- 读写：两个方向各写一个探针文件，对方读得到（本机写、第二台读，第二台写、本机读）。

`task-server/control check` 默认读服务 `/health` 的 `nfs` 栏最近一份结论（输出写明是多久之前判的），几秒内出结论；`control check --probe [--limit <秒>]` 现判这四项，每一项开始、结束各打一行，总时长不超过 `--limit`（缺省 60 秒），到点打出卡在哪一项、退 1；`/health` 的 `nfs` 栏同样报这四项，其中「读写」那一项（两向探针）是服务起来时那一次探针的结论，之后不再重做；服务起来之后只复核前三项、每分钟一次，转为不过的，服务直接发消息给名下有任务在排、在跑的会话（见「## 发消息给会话」）。

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
task-server/task confirm <号> --why '<…>'           # 跑满两小时的提醒核过范围与用时、确是要跑这么久：确认之后不再提醒（上面「跑满两小时提醒」）；放不过四小时硬上限（上面「四小时硬上限」）；开着 oversized-budget、fills-any-cap 提醒的任务核过范围、确要这么多的，同样用它记照跑（不看两小时的界，换过服务也认报过的那几种），记过的那一种不再再推
task-server/task confirm <号> --items <N> --tasks <M>  # 照确认单核过范围之后确认：两个数都与服务此刻算的相同才进队（照提交时刻排位置），对不上退 3、打服务算的确认单
task-server/task wait <号>                         # 取结局：完成时退出、末行 TASK_SERVER_RESULT …，退出码是任务的；记着会话收件地址的任务不必为了被叫醒起它（见「## 发消息给会话」）
task-server/task status [<号>]                      # 这个会话待确认、在排、在跑、完成了没人取的；给号的待确认那一条打此刻现算的确认单；排着的带「在等：…」，在跑的带已跑多久、日志多久没涨
task-server/task cancel <号>                        # 只许提交它的会话；回读到停了，同一次取消里删这条任务的编译目录与现场，打删了几份、删前删后多大
task-server/task log <号>                           # 最后一次尝试的日志（最后 200 行；--all 整份；--attempt <第几次> 取那一次；任务或那一次不在退 4）
task-server/task clean <号>                         # 删跑完的任务留下的编译目录与现场（只许提交它的会话、有了结局才删；没设 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的编译目录有结局就删了、现场在 wait 取走之后一小时删；第二台那一趟的编译目录 multi-host-run.sh 自己删；取消的不用它）
```

交任务的二次确认：`task submit` 收下的任务先记待确认（`awaiting-confirmation`），不进队、不建子任务、不起尝试；标准输出接着打服务回的确认单：整条命令、`SINGLEFS_RUN_SCOPE` 的四段原文、服务从命令里认出的项数 N（`--item`、`--check` 的每一格、点名的 `--test`、`--bin`、`--lib` 与 `--exact` 之后的测试名各几个；一个都没点的：整包测试、不带选条参数的门禁与入口、别的 cargo 写「整包、不指定范围」，别的写「范围由脚本里面定，服务算不出」）、崩溃用例数与域（命令里点名的 `crash-case:` 项数与 `--crash-expand-up-to` 的值；`merge-and-verify.sh`、`crash-point-replay.sh` 命令里都没写的写「范围由脚本里面定，服务算不出」）、这个会话此刻在排在跑的条数 M 与其中命令键相同或点的项交叠的逐条列号、内存上限与记录来源、交的时候就判得出的异常（预算贴着整台、按给的上限开并行，带那一种的修改意见），末行 `TASK_SERVER confirm-needed task=<号> items=<N> tasks=<M> timeout=<秒>`。核过范围对了写 `task-server/task confirm <号> --items <N> --tasks <M>`：两个数都与服务此刻现算的相同才进队，任务号是提交时刻的、照它排位置；对不上的退 3、打服务此刻算的确认单（这一会儿同会话别的任务起了或收了，数会变）。分片的父任务确认一次，子任务由服务建、不再要确认。满 30 分钟没确认的服务取消（结局 `cancelled`，说明以「没确认，已取消」起头），照结局发给会话。只有提交它的会话能确认；子 agent 不交任务，也不确认。

取消的不留产物（用户：「取消的任务要自己删掉产物 不要残留」「这个加入流程中 让task server去做」）：`task cancel` 回读到单元停了，服务在同一次取消里删这条任务（分片的连同子任务）每一次尝试的编译目录与现场目录（设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1` 的也删）；派到第二台的，第二台本地的 `builds/<标签>/` 与 `runs/` 下进程号、退出码、输出三个文件由 `multi-host-run.sh` 的 ⑥ 删，服务照那一次尝试日志里它打的「⑥ 删了第二台这一趟的本地目录 …（回读不在了）」认，没有这一行而清理回执还是 `not-started`、日志里也没有建了那一趟本地目录之后才打的行的，是还没建，不算没删成，上一次留着的经 `--remove-build-directory` 删。删前删后的大小逐份打在 `cancel` 的输出里、写进那条任务最后一次尝试的日志（第二台本地盘上的那一份服务这一侧量不到，照写）；回读不在了才算删了，没删成的逐份点名、`cancel` 退 5。

提交时警告（用户：「绕过的 那个要做成提醒 谁做错了严厉警告」）：照收照跑，服务当场打警告，进提交回复（客户端打到标准错误）、这条任务每一次尝试的日志头与 `status`。头一行「✗ 严厉警告：会话 <号> 交的任务 <号> 绕过了服务给的编译目录，取仓根 target/ 下的二进制（…）…」，第二行「→ 怎么办：…」。判的是：命令里的词落在仓根 `target/` 下，或命令位置上起的本仓脚本正文里有指向仓根 `target/` 的路径字面（前面接仓根变量、`$(dirname …)/../..`、`$(git rev-parse --show-toplevel)`，接 `target/release/`、`target/debug/`；在仓根起的另认裸写的）。服务给的编译目录、调用方给的 `CARGO_TARGET_DIR`、草稿与临时目录下的 `target/`、注释、弄坏开关那一支与自证那一支（带 `--selftest` 起的、名字带 selftest 或以 cells 收尾的函数体）不算。判法在 `task_server.py` 文件头 ②「提交时警告」①。另一种（同一节 ②）：交任务时会话名下有任务的结局满 30 分钟没交给它（服务发的消息没送到，也没起 `wait`），照收照跑，同样进三处，头一行「✗ 严厉警告：会话 <号> 有 <N> 条任务跑完了没人取（<号>、…）：…」，第二行「→ 怎么办：task-server/task wait <号> 取走（…）…」；没有会话的人手任务不判。

`wait` 自己报任务要处置的提醒，共十五种（`task_server_lib.py` 的 `ATTENTION_KINDS`）：排队过久（`queued-too-long`），排满 `QUEUE_ATTENTION_MINUTES`（30 分钟）；清理没收尾（`cleanup-pending`），命令已退出而收尾回读还有进程或读不了（本机单元与 scope、第二台那一趟的清理回执），服务保留资源占用；分片的父任务推不动（`shard-advance-failed`）；日志长期不涨而服务没停它（`quiet-log-busy`），日志满 `QUIET_ATTENTION_MINUTES`（30 分钟）没涨、服务因为 CPU 在涨或读不到 CPU 而没停它；日志不涨而有进展的不报：日志满 10 分钟没涨之后服务每隔 5 分钟读一回这一趟进程树里各进程打开写着的文件（顶层日志不算；两台都读，第二台经同一条 ssh 通道起同一段），写着的文件在涨（测试进程把输出另存在运行目录的 `record.log` 这类）算有进展，多久没涨从最近一回读到进展起算，在跑进度那一条同时写出是哪份文件在涨；按卡在 IO 上免停的不报这一条，由 `io-wait` 报；卡在 IO 上而服务没停它（`io-wait`），日志没涨的每一轮都有进程在 D 状态、连着满 `IO_WAIT_ATTENTION_MINUTES`（30 分钟），依据写最近一轮 D 状态的进程数与本机此刻 `/proc/pressure/io` 的 some avg60；本机读不到在跑尝试的单元状态（`unit-unreadable`），对账连着读不到满 60 秒，之后每翻一倍再报，依据与出路写在下面「读不到单元状态」一节；跑满两小时没确认（`long-run-unconfirmed`），判法与确认写在上面「跑满两小时提醒」那一段。另有三种只报有结局的任务：改主工作区的命令（`apply-writer-patch.py`、`hook-swap.py`、`stage-mine.py`）以撞上限、被挤掉、第二台没跑成、撞顶、取消、卡住被停、对不回来或被信号杀收尾（`workspace-write-interrupted`），可能写了一半，`next=` 点名要核的文件或补丁目录（用户：「这个问题很严重 因为可能会导致补丁没有正常写入」）；`apply-writer-patch.py` 有结局文件 `apply-outcome.tsv` 的，服务照它的 `--read-outcome` 判，打全了、已退回的不报，带 `--dry-run` 的不报；换钩子的命令（`scripts/gate/hook-swap.py`，不带 `--feed-only`）退 7（上一趟换上没写完成、拒绝再换，一个字节没写）的，报给被挡的这个会话（`workspace-write-blocked`，只推一回）；出结局时用时超过 7200 秒又不在 `.claude/long-run-split-exempt` 里的，报 `long-run-split`（只推一回，出路指到 `.claude/rules/long-run-split.md`）。另有五种（用户：「既然加了给对应对话发消息的功能 就就该把异常和修改意见交给对应的对话」），只提醒、照排照跑，依据写量到的数、`next=` 写修改意见：预算贴着整台（`oversized-budget`），排队那一刻判出它在某一台上的记账不小于那台的整片界（可用上限的 85%）而不超过可用上限，依据写各台的记账、可用上限与记账从哪来（记录峰值几次、没有记录的缺省、钉值），钉了上限的照样报，修改意见是核实范围、分片与同时起的个数；预测峰值、上限与记账都收到判的那一台的可用上限为止（钉值、记录、没有记录的缺省、撞上限重排一样收），记录比整台大的不判永远放不下：别的台装得下就派那一台，哪台都装不下就照收到的数起跑，收到最大还撞上限的才打回 `memory-cap-at-maximum`；跑得久而核没用起来（`long-run-idle-cores`），一次尝试跑满 600 秒、到此刻平均在用核数不到给的线程数的 24/31（线程数 31 时即不到 24 核），每次尝试只报一次、不再推，依据写「给 31 线程、平均只用 x 核」，修改意见是核实资源运用是否合理、脚本能不能并行或切片、范围是否点多；按给的上限开并行（`fills-any-cap`），同一命令键最近三次跑成的峰值都不低于各自上限的 90%（撞了上限的那几次不算：撞的那一次峰值本来就贴着上限，照上限一档档撞上去的不是按上限开并行），修改意见是「这条命令按给的上限开并行，核实」；连着撞上限还在重排（`repeated-cap-hits`），同一条任务末尾连着三次尝试都撞了内存上限、服务还在乘 1.5 重排，每条任务只报一回（第四次再撞不再报），`wait` 那一行另带 `loop_leak=`、推给会话的正文另起一行写「检查代码有没有逻辑循环或内存泄漏：…」那一段（上面「撞上限那一刻的内存明细」那一段的后半），修改意见是照那一段逐类核代码，有循环或泄漏的取消、改了代码再交，规模本来就这么大的不用管；线程总数远超登记数（`threads-over-registered`），一次尝试的线程总数连着三轮超过登记数的两倍、其中可跑的也超过登记数、占线程最多的那一个进程自己也超过登记数的两倍，依据写线程数、可跑的个数、登记数与占线程最多的进程名，只起一堆在睡的线程的编译器、链接器不算，每次尝试只报一次、照跑不停它。每一种（连同提交时拒、提交时警告、各种结局、疑似卡住与结局没人取）怎么判、动作、修改意见与没处置时怎么办，只照 `task_server_lib.py` 文件头「异常表」那一行：`next=` 的头一段就是那一行的修改意见，点名这一种要核的东西（例：`io-wait` 核那台的盘与 NFS、`/proc/pressure/io`；`cleanup-pending` 核留下的进程命令行、脚本有没有不等子进程就退），后面接这一回的具体数。服务把这条任务（分片的连同子任务）的这几样记成「要处置」的提醒，正在等它的 `task wait <号>` 读到一条新提醒就打一行 `TASK_SERVER_ATTENTION task=<号> type=needs-attention kind=<种类> [reminded_after_seconds=<秒>] basis=<依据的数> why=<原因> next=<出路>`（`why=` 超过 300 个字符的截到那么长、尾巴接「…」，全文在服务发给会话的那一条的「原因：」一行），然后退 230：任务照旧在排或在跑，服务没动它，按 `next=` 处置，再起 `wait` 接着等。任务已有结局的（改主工作区的没正常结束这类）这一行打在 `TASK_SERVER_REASON` 之前，退码仍是任务的。同一条任务同一种提醒、同一道提醒门槛只叫醒一次，重起 `wait` 不再叫，到下一道门槛（起点的两倍、四倍……）才再叫，时长判的那几种在 `kind=` 之后写 `reminded_after_seconds=<这一回到的门槛，秒>`；报到的门槛记进服务的库，服务重启后照库认，同一道门槛不再报；排队过久那一种，排队原因还没判出来（服务刚重启、这一轮还没判到它）的那一轮不报，等判出来那一轮再报。这几种提醒服务另直接发消息给提交它的会话（见「## 发消息给会话」），与 `wait` 同一份记账：发过的 `wait` 不再报，`wait` 报过的不再发。服务健康不归 `wait`：服务自己盯，同样发消息。

命令里不套包装、不写资源参数：`run-with-memory-cap.sh <上限>`、`host-dispatch.py … --`、`multi-host-run.sh … --`、`nice`、`timeout`、`capped.sh`、`two-logs.sh`、`gpu-quiesce-run.sh` 与脚本自己的 `--memory`、`--memory-cap`、`--threads`、`--peer`、`--local-only`、`SINGLEFS_THREAD_CAP`：这几样由服务做，写了当场拒并点名去掉哪一样。白名单外的环境变量当场拒并点名；命令真要它的写成 `task-server/task submit -- env <名>=<值> <命令…>`。退役的变量（`SINGLEFS_HEAVY_TESTS`、`GATE_MUTATION_START_OVER`）设在环境里、写在命令里（`env` 后面与命令前面的 `名=值`）都当场拒，拒绝说明点名它、写出路；名单是 `task_server_lib.py` 的 `RETIRED_VARIABLES`。

派不到第二台的三种提交时当场拒（用户：「改成提交时当场拒（推荐）：不再悄悄改派本机：碰到这三种就在提交时报错、点名是哪个路径或哪一族、出路写怎么改，任务不跑。」）：`task submit` 退 3，打那一句与出路，任务不进队、不跑。一是工作目录与命令里的绝对路径（只认整个 shell 词：以 `/` 起头的整个词、`名字=绝对路径` 与 `--选项=绝对路径` 的值、`bash -c` 脚本词照 shell 词切开之后的每一个；别的词中间夹着的 `/`（参数值里的「（接的是 /）」这类）不算）有不在多机配置 `NFS_SHARED_ROOTS` 任何一段下的，点名那个路径，出路是改放某一段下；`/dev/null`、`/usr/`、`/bin/` 这类两台相同的系统路径不算；`/tmp` 下的同样拒，那一句另说重启会清 `/tmp`。二是交给命令的 `SINGLEFS_DATA_ROOT`、`SINGLEFS_WORK_ROOT` 有不在共享根下的，点名变量与值，出路是配多机配置。三是命令点到登记表 `.claude/gate.d/verdict-library-scripts.tsv` 里带判定库的一行、而那一行第三列还没写 `prune-server` 的，点名那一族，出路是那一族接进 prune-server、把第三列翻成 `prune-server`；登记表读不了、写坏了的同样拒。

临时 prune-server 的包装（`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>`）只给脚本自证用：剥出的原命令里有会编的 cargo 的（认法与服务定编译目录的是同一套），提交时照上面同一种形态当场拒：`task submit` 退 3，打那一句（点名原命令）与出路（去掉包装重交），任务不进队、不跑。剥开只用来判，收下的照旧整条执行；编译目录只照外层那一条给，服务不剥包装去迁就。不带 `--server-binary`（命令前的赋值与任务环境里也没给 `PRUNE_SERVER_BINARY`）的包装自己要在服务给的编译目录里现编 prune-server，服务把外层这一条按会编的算：派本机、派第二台都给编译目录；带了 `--server-binary` 的不编、不给。验证包的 `cargo test` 对真服务判、不套这层包装。

不点名包的全仓编译提交时当场拒（用户：「8105打回 让他重新写重新确认范围」；任务 8105 不带 `-p` 的 `cargo build --offline --all-targets` 预测 49.0 GiB）：提交的这一条在命令位置上直接起的、会编的 cargo（`build`、`check`、`clippy`、`test`、`run`、`bench`、`doc` 与它们的单字母别名；`bash -c` 与 `cd` 跟着走）带 `--workspace` / `--all` 的拒；没带 `-p` / `--package`、照 cargo 找清单的走法（起的目录往上第一份 `Cargo.toml`）落到虚拟清单（只有 `[workspace]`、没有 `[package]`，仓根那一份就是）的拒，起的目录认不出、清单读不了的同样拒。`task submit` 退 3，打那一句（哪一条 cargo、在哪个目录起、照哪一份清单）与出路：点名要编的包 `-p <包>`（验证包 `-p singlefs-verification`），整仓要跑的走 `scripts/gate/merge-and-verify.sh` 或提交时的门禁。落到包自己的清单的（`scripts/experiment-apparatus/`、`prune-server/` 这类自成工作区的包，或在一个成员的目录里起）只编那一个包，照收；入口脚本与门禁里面起的 cargo 不判（服务只看提交的那一条）；`cargo fmt`、`metadata`、`tree`、`clean` 不编，不判。

## 发消息给会话

用户：「不是你通知 而是 task-server 通知发起的对话」「包括报错 长时间没有响应 都可以这样通知」。`task submit` 在会话的 Bash 里跑，读环境里的 `CLAUDE_CODE_MESSAGING_SOCKET`（会话的收件 socket）与 `CLAUDE_CODE_MESSAGING_TOKEN`（会话令牌），随任务交给服务，服务记在任务上；令牌不进日志、`status` 与 `/tasks/<号>` 的回话。读不到的当场拒（退 3），出路是从 Claude Code 会话的 Bash 里交（子 agent 不交任务）。收件 socket 不照交任务时的那一个：每次发之前服务按任务记的会话号读会话登记（`~/.claude/sessions/*.json`，一份一个进程，只读 `sessionId`、`messagingSocketPath`、`pid`），取会话号相同、进程还活着的那几份里启动最晚的一份的 `messagingSocketPath`；令牌取交任务时带的。会话重启换了 socket 的，照新的发；找不到的（会话已关）照下面「发不出去」写告警。

服务在这几件事上往那个 socket 发一条消息：

- 任务有了结局（`wait` 会打 `TASK_SERVER_RESULT` 的每一种）：只发一次；发出去了服务记成已取（同 `wait` 取走），会话已经用 `wait` 取了的不发。
- 要处置的提醒（`task_server_lib.py` 文件头 `ATTENTION_KINDS` 那十五种）与疑似卡住：同一条任务同一种提醒同一道门槛只发一次，与 `wait` 同一份记账；这份记账都在库里（要处置的提醒在任务行，疑似卡住报过的段、服务健康发给了哪几个会话、记账漂移推到哪道门槛在换服务不丢的那几份），服务重启后同一件不再发。出结局时用时超过 7200 秒又不在 `.claude/long-run-split-exempt` 里的推一回 `long-run-split`（按片或按整条任务的用时判），出路指到 `.claude/rules/long-run-split.md`；换钩子退 7（被上一趟没写完成的挡住）推一回 `workspace-write-blocked`，报给被挡的这个会话。一次尝试的线程总数连着三轮超过登记数的两倍、其中可跑的也超过登记数的，推一回 `threads-over-registered`（依据写线程数、登记数与占线程最多的进程名；照跑、不停它；只起一堆在睡的线程的编译器、链接器不算）。时长判的每翻一倍再发一回；门槛不随时长涨的（`oversized-budget`、`fills-any-cap`、`long-run-unconfirmed`、`cleanup-pending`、`shard-advance-failed`）发过之后满 30 分钟还在的再发一回、之后每翻一倍再发，第一行写「没处置、第 <回> 回再推」，正文另写之前推过什么、此刻还差什么。
- 结局没人取：父任务的结局满 30 分钟还没交给提交它的会话（发结局的那一条没送到，也没起 `wait`）的，再推一回、之后每翻一倍再推，第一行「task-server：结局（任务已结束，这条结局还没人取）任务 <号> <中文名>（<种类>，退 <码>；第 <回> 回，上一回在 <日期>）」，推出去了记成已取。
- 交了没确认、满 30 分钟被服务取消的：照结局发，正文带「没确认，已取消」。
- 任务之外的进程空转：两台每 5 分钟扫一次，不在任务 slice 里、累计 CPU 满 30 分钟且占已跑时长一半以上的进程，只发给它所属的会话，还在的每 30 分钟再报；认得出所属会话的本机进程报两次还在，服务按号停掉并告诉那个会话；认不出发起会话的（第二台的、发起的会话已退出的）直接终止，只写告警；会话进程与其祖先、systemd 单元与系统服务、sshd、编辑器这些不碰。
- 服务健康：库写不进、派活判排头能起却一直没起成、看进展报放得下却一直没起、服务的单元不在、NFS 共享前三项不过、第二台用不了满 180 秒。发给每个名下有任务在排、在跑的会话，同一样对同一个会话只发一次，好了再坏再发。服务自己挂了发不了：`task wait` 连续连不上满 60 秒就退出报错，`task submit` 连不上当场报。

正文第一行先写类别与任务此刻的状态，只看第一行就分得出：「task-server：结局（任务已结束）任务 <号> <中文名>（<种类>，退 <码>；第 1 回）」、「task-server：提醒（任务还在排 / 还在跑，不是结局）任务 <号> <中文名>（<种类>；第 <回> 回，上一回在 <日期>）」、「task-server：疑似卡住（任务还在跑、服务没停它）任务 <号> 疑似卡住（suspected-stall，第 <段> 段；第 1 回）」，或「task-server：服务健康 <哪一样>」；每个种类带中文名，英文种类名在括号里，重推的标第几回与上一回在哪一天。后面照 `wait` 打的那几行写依据、原因、出路，「长跑已确认」这类确认标识的行在依据与出路之后，再是日志，末尾是 `wait` 那一行原样，只写事实与出路。发不出去（socket 不在、连不上、会话已关）不重试到死：服务日志与告警记录写一条 `inbox-unreachable`，那条任务最后一次尝试的日志接一行 `TASK_SERVER_INBOX` 起头的，任务照旧。收方空着时 Claude Code 照这条消息起新的一轮；收方是跳过权限确认的会话时，消息扣下等用户批准。

`task wait` 留着：照旧取得到结局与提醒，会话不必再为了被叫醒而起它。`task-server/control watch` 已删（退 2、打出路句）；人手查一次服务健康用 `control check`。

行的写法：先发一行 `{"type":"auth","token":"<会话令牌>"}`，再发一行 `{"type":"user","message":{"role":"user","content":"<正文>"}}`；两行备好再连、一次写完（Claude Code 30 秒里收不到一整行就关连接）。来源：收件 socket 与认证行见 Claude Code 文档 https://code.claude.com/docs/en/cross-session-messaging.md「The session's inbox socket」一节；user 那一行的写法文档页没写，取自 Claude Code 二进制自己的调试日志里「[uds-messaging] Inject messages」那一句给的示例。文档页没写行格式，升级 Claude Code 之后要复核。

## 源文件头的声明

服务按源文件头前 60 行里的几行声明认一条命令要什么，调用方照旧不写额度。声明由管那份入口或那个包的会话加，服务只读：

| 写在哪 | 写法 | 服务怎么用 |
|---|---|---|
| 入口脚本 | `# task-server-shards: <参数>…`、`# task-server-resumable: <参数>…` | 分片、续跑（`task_server_lib.py` 文件头「分片」「续跑」） |
| 入口脚本 | `# task-server-compiles: yes` | 会编：命令那一层认不出 cargo 的门禁入口也照会编给编译目录（本机与第二台都给 `CARGO_TARGET_DIR`，撞上限重排接着用上一次那一份；分片的计划、各片、合并照父任务）；写了别的词打回。单独 `--selftest`、带 `--list-items` / `--help` / `-h` 的不算 |

领卡、要卡的声明（`# task-server-card-lease:`、`# task-server-gpu-cards:`、`# task-server-gpu-arguments:`）服务不认：提交的命令与 `bash -c` 里命令位置上的每一条起的脚本，文件头前 60 行写了其中一种的，提交时当场拒（`task submit` 退 3，异常表 `card-declaration`），说明头一句是「卡归 GPU 批处理验证服务，任务不领卡」，点名那份脚本与写了的那几种，出路是删掉那一行、要卡的那一段改成往判定库写待判行交 GPU 服务判（见「## 卡」），再重交。判法在 `task_server_lib.py` 的 `card_declaration_problem`。

定额度（服务自己库里的尝试记录，`task_server_lib.py` 文件头「资源记录」；只有这一条定法，脚本与调用方不报内存量，入口文件头没有报内存的声明）：
- 同一命令键（同类命令）库里有跑完的尝试的，取近 5 次的最大（跑成的取量到的峰值，撞了上限的取那一次的上限），上限与记账都等于它，不乘 1.5；父任务与同一趟兄弟片的记录不借（分片的片的键只带片号、不带片数：片数变了同一片号落在同一把键上，定额度只认片数与这一趟相同的记录）。这把键（连同片数）自己没有记录的照没有记录的命令定，别的键的记录不借。只选范围的选项不进键：`--item`、`--only`、`--skip`、`--exact`、`--fingerprint-of`、`--crash-expand-up-to` 连同后面的值，`cargo test` 的测试名筛选词，点名格与条目的环境变量（`*_SELFTEST_CELLS`、`GATE_MUTATION_ROW_SELECTION`）都去掉，同一入口点不点、点哪几条共用一份记录；门禁 code-tooling 的 break-switch-replay 与 crash-point-replay 的 crash-case-replay 两格点名的 `--item` 照旧各条各一把键（`scripts/hosts/memory_peaks_lib.py` 文件头「键」③）。脚本永远不报自己要多少内存，服务也不认这样的行。`cargo test` 同包同特性同档归一个键：键里只留包（`-p`）、特性（`--features`、`--all-features`、`--no-default-features`）与档（`--release`、`--profile`），点名的测试目标（`--test`、`--lib`、`--bin`、`--example`、`--bench`）与 `--` 之后的用例筛选不进键，点名不同目标的照同一份记录起步。被取消、命令自己退非 0、卡住或到四小时被停的那几次量到的峰值只当下限：比窗口里最大的还大才抬上去，窗口空着的比 4G 大才照它起，小的不拉低（下一次不再从撞过的低档重撞）。能升能降：窗口里刚撞上去的那一档马上看得到，更早的出了窗口就不再算，后面几次都跑得小，起点跟着降。库里没有这把键跑完的尝试的从 4G 起，撞了上限乘 1.5 重排（4096M → 6144M → 9216M …），命令自己打的「要多少」行一律不认。有真实峰值记录的不兜到 4G，只设 1 GiB 的下限：记录的峰值只算进程内存（`anon` 加 `shmem`），上限还要装读文件带进来的页缓存，起步的上限与重排的上限都不低于 1 GiB，记账照这个上限记（任务 11127 只读一个文件，峰值 5 MiB，上限 3M、5M、8M 都退 250）；记录比下限大的照记录；钉值不受这个下限管，低于下限也照钉值；下限不超过那台能给的最大上限。
- 撞了上限重排时，上一次的编译目录不删，重排的那一次还在同一台就接着用（推的：cargo 照指纹复用已编完的 crate，省多少没量过）；换了台的在那一次起的时候删上一次那一份。两台同一段：本机经 `CARGO_TARGET_DIR` 指过去；第二台的尝试带 `--keep-build-directory`，`multi-host-run.sh` 跑完留着 `builds/<标签>/`，重排的那一次带 `--reuse-build-directory <上一次的标签>` 接着用，换台或有了结局时服务经 `multi-host-run.sh --remove-build-directory <标签>` 删。

## 暂停与放开

换 prune-server 的二进制、清判定库旧行这类活要一段别的会话的任务都不在跑的空档，而这类活自己常要经 `task submit` 交命令。`control stop` 做不到：服务停着时谁都交不进任务，正在 `task wait` 的过 60 秒退 78。要空档就用暂停，并放行做活的那个会话（它交的任务暂停着也照常起）：

```bash
python3 task-server/control pause --reason "清判定库旧行" --allow-session <做活的会话>   # 暂停：别的会话的任务不起；做活的会话交的照常排、照常起；提交照收，status / wait / log / cancel 照答
python3 task-server/control check                                                          # 反复看，到打出「…、放行会话 <号>、在跑 N 个、别的会话在跑 0 个」为止
# …做要空档的活（做活的会话经 task submit 交命令）…
python3 task-server/control resume                                                         # 放开：排着的照常起，放行一并清掉
```

- 做活的会话的号：在那个会话里跑 `python3 task-server/task status`，第一行「会话 <号>：…」那个号（就是交任务时服务记下的会话，`status` 里每条任务的会话同一个字段）。放行的是会话，不是某一条命令：那个会话已经排着的、之后交的、分片的子任务、撞上限重排回队的都照常起；别的会话的、没有会话的人手任务照旧排着不起。放行会话排着的任务 `status` 照常写它真在等什么，别的会话排着的写「在等：服务暂停中（原因：…）」。
- 已经暂停着再 `pause --allow-session <另一个会话>` 退 0，换放行的会话（原因与起点不改）；不带 `--allow-session` 的 `pause` 不动放行；`resume` 把放行一并清掉。要一段什么任务都不在跑的空档：不带 `--allow-session` 地 `pause`，等 `control check` 报在跑 0 个。
- 暂停着，排队线程不起任何新尝试（放行的会话交的除外）：分片的子任务、撞上限重排回队的都不起；在跑的照跑到结局（结局照写、该重排的回队，回队的不起）。`pause` 与起尝试共一把锁，`pause` 退 0 时正在起的那一条已经记下，`control check` 的在跑数不会再被挤进一条。
- 暂停态连同放行的会话记在服务的库里（带签名，改库里的放行会话按校验不过算、不放行谁），服务重启后照旧暂停、照旧放行，直到 `resume`；`pause` 已经暂停着的退 0、不改原因与起点。
- `control check` 另打一行「暂停中（原因：…，起于…）、在跑 N 个」，放行了会话的写「…、放行会话 <号>、在跑 N 个、别的会话在跑 M 个」（M 是在跑的任务里不是放行会话交的），仍退 0，也不判「派活判排头能起却没起成」、不看进展；`/health` 带 `paused` 与 `pause`（含 `allow_session`、`other_running`），`ok` 不变；排着的任务 `status` 写「在等：服务暂停中（原因：…）」。
- 暂停着服务照样盯自己的健康（库写不写得进、单元在不在、NFS、第二台），查出毛病照样发消息给名下有任务在排、在跑的会话。
- 只许本机起服务的那个用户发 `pause`、`resume`，认法同提交、取消：服务看调用方的 uid 是不是它自己的。
- 空档的活做完一定要 `resume`；暂停着的服务平时看不出异常（`check` 退 0），忘了放开排着的任务会一直等。

## 读不到单元状态

对账读不到一次尝试的单元状态（`systemctl --user show` 起不来、超时或没打 `ActiveState`）时判不出单元在不在。留了 `result.json` 的照它的退出码收尾；没留的服务不杀、不收尾、不记结局，尝试照记 `running`，不设到点就收尾的时限，改用不经 systemd 的办法核这一次还有没有进程：本机读起跑标记记的 runner 进程号在 `/proc` 里在不在，单元的 cgroup、`attempt_runner` 写的 `scope` 文件点名的 cgroup 与带这一次尝试标签的内存包装 scope 的 `cgroup.procs` 空不空；派到第二台、清理回执还是 `pending` 的，第二台经同一条 ssh 通道跑同一段，读 `multi-host-run.sh` 记的 `runs/<标签>.pid` 与 `runs/<标签>.scope`。cgroup 都读得到且空（目录已收掉也算空）、记下的进程号都不在，满 10 秒宽限才照没留结果记 `launcher-lost`；有一样核不出（cgroup 读不了、起跑标记不在、清理回执读不到、第二台连不上），一直等。

读不到的那一段：连着满 60 秒给发起任务的会话推一条 `unit-unreadable`（要处置的提醒，依据写尝试派在哪台、本机读不到哪个单元的状态、核到的进程号与 cgroup），之后每翻一倍再推，同一道门槛不重推；`/health` 回不健康，`problem` 写本机读不到哪几次尝试的单元状态，`unit_unreadable` 栏逐次列出（核到还有进程在、进程都不在、核不出、还没核完），`control check` 照它退 78。修好本机的用户级 systemd 之后服务照常判。判法在 `task_server.py` 文件头 ①「读不到单元状态」。

## 卡

卡归 GPU 批处理验证服务（`gpu-verification-server/`，`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）：它照多机配置开卡、一直持着，开卡之前的清场、查卡与判都归它。这个服务只调配 CPU 与内存：不按命令或脚本名字认谁要卡、不分卡、不领卡、不清场、不收卡，派活只看内存、CPU 闸与编译目录占盘；起的命令外面不套清场包装，服务自己环境里带着的选卡变量（`task_server_lib.py` 的 `CARD_CHOICE_VARIABLES`：`GPU_CARDS_BY_PRIORITY`、`CUDA_VISIBLE_DEVICES`）不交给尝试。入口文件头写了领卡、要卡的声明的提交时就拒（上面「## 源文件头的声明」）。两个服务只经判定库（prune-server 那一份 RocksDB）连：任务只跑录入、往判定库写待判行、读结论。判法在 `task_server.py` 文件头「卡」。

## 自证

`python3 task-server/task_server_lib.py --selftest`：每格一份真的 uvicorn 挂在临时 socket 上，内存包装、第二台、`control` 用的 systemctl 换成假脚本，尝试的 scope 换成假目录，不起编译、不连第二台、不上卡。它跑的全是模拟世界，主 agent 直接起、不经服务排队（钩子名单 `.claude/hooks/lib_project_code_runs.py` 的 `MAIN_AGENT_TOOLS_RUN_DIRECTLY`）；`TASK_SERVER_SELFTEST_CELLS=<格,…>` 只跑点名的格。各格并行的路数取每台默认重活线程数的登记值（`scripts/hosts/heavy_threads.py` 现读），取不出（登记文件不在、写坏，`SINGLEFS_THREAD_CAP` 不是正整数）整份判红、不退回按核数。弄坏开关 `TASK_SERVER_BREAK=<项>` 的表在 `task_server_lib.py` 文件头。

自证与开关趟起的尝试单元随外层停（`task_server.py` 文件头 ③「套在外层单元里的服务」）：服务在自证里、或自己起在一次尝试里时，照 `/proc/self/cgroup` 认出自己所在的用户级单元，起每个尝试单元都带 `BindsTo=`、`After=` 那个单元；外层任务一停（取消、卡住被停、撞上限），systemd 当场停这些尝试单元，不靠它们自己超时。两台同一段。真服务（独立的服务单元）不带：尝试单元要在服务重启之后接着跑。
