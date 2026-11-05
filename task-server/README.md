# task-server

**中文** · [English](README.en.md) · [日本語](README.ja.md)

异常退出后的资源占用、远端清理回执与诊断日志见[任务收尾与 GPU 阶段日志](cleanup-protocol.md)。

singlefs 的统一任务服务：一切跑项目代码的任务（轻、重、CPU、GPU）都经它起，同一个队、按到达次序、不另设延迟。任务只由主 agent 交（客户端 `task-server/task`，人手同样用它）；子 agent 只用 `status`、`log` 看结局；挑机器、定内存额度、要不要 GPU、分哪张卡都由服务定，提交时不收这几样。

服务是唯一读写库的进程：FastAPI 跑在 uvicorn 上，作为用户级 systemd 服务 `singlefs-task-server.service`，只开 unix socket（`${XDG_RUNTIME_DIR}/singlefs-task-server/server.sock`，目录 700；全路径至多 107 字节，超了服务、客户端、`control` 都退 78，点名那个路径、它的字节数与上限），不开 TCP。库是内嵌的 SQLite（WAL），放在 `~/.local/state/singlefs/task-server/`，每一行带 HMAC 校验列。内存准入只在服务里判一处：放得下才起，放不下的留在库里排着；起任务仍套 `scripts/hosts/run-with-memory-cap.sh`（本机，设了「已由 task-server 准入」的开关，只做每条上限与 slice 总上限的内核层兜底）或 `scripts/hosts/multi-host-run.sh --peer`（第二台）。

| 文件 | 做什么 |
|---|---|
| `task` | 客户端：`submit [--cwd <目录>] -- <命令…>`、`wait <号>`、`status [<号>] [--json]`、`cancel <号>`、`log <号>`、`clean <号>`；用法与退出码在文件头 |
| `control` | 装与查：`install [--dry-run]`、`check`（服务的 `/health` 只报三样：库写不进；派活判排头能起却连着三轮没起成，判不能起的——等卡、等负载、交替封组这类——不报；看进展：某台照公式放得下排着的最小那条、却 10 分钟一条都没起，兜派活自己判错）、`start`、`stop`、`watch`、`clear-queue`（服务停着时清掉库里的任务与尝试两张表，先备份库，资源记录不动）、`pin-cap <命令…>` / `unpin-cap <命令…>`（经服务给一条命令钉上或撤掉内存上限的钉值，回读过才退 0）、`pause [--reason <一句>] [--allow-session <会话号>]` / `resume`（暂停与放开排队：暂停着照收提交、不起新任务，只有放行的会话交的照起，见「## 暂停与放开」） |
| `task_server.py` | 服务（`serve`）：接口、排队、准入、起单元、对账、重试、分片、资源记录；判法在文件头 |
| `attempt_runner.py` | 每一次尝试的单元里起命令、记退出码与资源（内存峰值取按轮采到的与内存包装退出时读的里大的）；编译目录任务有了结局由服务删 |
| `task_server_lib.py` | 共用的那一份：路径、依赖、错误类型与退出码表、重试、资源记录、编译目录、分片、环境变量名单、身份；整套的自证 `--selftest` 与弄坏开关表在它文件头 |
| `task_server_selftest.py` | 自证的各格（由上一行起） |
| `singlefs-task-server.service.in` | 用户级单元模板 |

没有自己的配置文件：调度资源照仓根的多机配置 `multi-host.env`（模板 `multi-host.env.example`）读；socket、库、日志、签名密钥、编译目录用 `task_server_lib.py` 文件头「路径」写的固定缺省值。必须依赖：fastapi、uvicorn、`multi-host.env`（含 `NFS_SHARED_ROOTS`、`SHARED_DATA_ROOT` 与 `HOSTS`，写法见「## NFS 共享」的「多机配置写哪几键」）、NFS 共享配好（见「## NFS 共享」）、服务通。

挑机器按余量均衡：不要卡的任务在派得过去的几台里挑放得下的，放得下的不止一台时挑放下之后剩余占那台总上限比例大的，一样大取本机；撞上限重排不钉台。分片的各片照计划先去第二台，放得下就不按均衡改台；计划、各片与合并都不钉台，计划里写了 `pin` 的整件记 `shard-plan-refused`（片不钉台，哪台由服务挑）。第二台在同一棵共享的树里跑：仓、工作根 `~/.local/share/singlefs/work/` 所在的共享数据根与会话记录目录经 NFS 两台挂在同一绝对路径（多机配置 `NFS_SHARED_ROOTS`，下面「NFS 共享」一节），派第二台的那一次经 `scripts/hosts/multi-host-run.sh --peer` 在任务的工作目录、同一个绝对路径里直接起，不拷树、不送、不取回、不改写路径，写出的文件两台都读得到；编译目录由 `multi-host-run.sh` 在第二台本地盘上这一趟新建（多机配置 `BUILD_ROOT_BY_HOST` 里第二台那一项下的 `builds/<标签>/target`，不在任何共享根下）、命令结束或被停都先删掉（会编的命令撞上限重排时留着给下一次，见下面「峰值起步」），服务不设、只记下它在哪（`task status` 看得见），`task clean` 不管它。派不派第二台只看第二台此刻用不用得了与余量；派不过去的三种在提交时就拒（下面「## 提交写法」）。没有只派一台的名单（用户：「【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：点到哪个目录、哪个入口的命令都派得到两台，哪台由余量定；派到第二台跑红的照红，由脚本那一侧修，服务不替它挡。改主工作区的工具（`hook-swap.py`、`apply-writer-patch.py`、`stage-mine.py`、`scripts/kb/` 下的、不带 `--skip-apply` 的 `merge-and-verify`）与带 `--write` 的派得到第二台（改的就是同一棵树），撞上限这类本会重排的结局照旧不重排、打回提交它的会话。分片的计划目录与各片的产物目录在共享工作根下的 `task-server-shards/`，两台照同一个绝对路径读写。服务不认合入锁：主工作树正在合入时照常排、照常起。一轮至多在第二台起 4 条（一齐起太多 ssh 会被重置），起满了 2 秒后再排一轮；第二台的结局照 `multi-host-run.sh` 打进日志的那条命令的结束行（`MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>`）定：有这一行按命令自己的退出码定，命令自己退 255 记 `command-failed`、不重排；没有这一行而退 255 才是第二台没跑成（连不上、ssh 被重置），记 `peer-run-failed`、原样重排，至多 2 次。排头放不下时后面放得下的先起（插队）；没满两小时的照整片与碎片按台交替：每台可用上限（总上限减那台此刻的余量）的 85% 为界，记账到界的是整片，别的是碎片；碎片按队列先后装成一组、装到再放一条就超过界时，有整片在等就封组、不再放碎片，整片放得下就起（不等那台腾空），整片起了回到下一组碎片，交替挡住的排队原因带「在等这台腾空」。一条整片只在一台上等腾空、只封那一台：上一轮已为它封了组的接着等那台，不然挑此刻离放得下最近的那台（它的记账减那台照「配置的总上限 − 余量 − 在跑的上限之和」还剩的最小）；别的台照常起碎片。要卡的整片只在空卡凑得够它要的张数的台上算，卡被占着的台不为它封组。`/health` 判排头放不放得下与派活同一个判法（内存、编译目录占盘、CPU 闸、卡的张数）。封着组时插空：一条碎片在排头整片起来之后照「配置的总上限 − 余量 − 在跑的上限之和」那台还放得下的，不看时长就起；会占排头那部分的，预计用时不超过那台在跑的尝试里最长的剩余用时（挡路时长）才起，没有预计用时的照旧封着。预计用时：`scripts/crash/crash-point-replay.sh`、`.claude/gate.d/formal-verification.sh`、`.claude/gate.d/crates-mutation-replay.sh` 点了 `--item` 而不分片的，服务先起一条估算子任务（同一条入口接 `--list-items --with-keys`，读每项一行 `ITEM_PLAN item=<项名> to_judge=<要现判的条数> reusable=<能复用的条数>`），它有了结局才进队；每次跑完照日志里每项一行 `ITEM_DONE item=<项名> seconds=<秒> judged=<现判条数> reused=<复用条数> exit=<码>` 记每项「每现判一条几秒」与「每复用一条几秒」（记在服务自己的库里，不进 prune-server），预计用时是各项条数乘速率之和，缺一项速率就没有预计用时；估算没成的照没有预计用时排。别的命令的预计用时是同一命令键（与内存记录同一把键：路径、条目照词折过，`--check` 的格名照留）最近一次跑成的用时；分片的计划、各片与合并三步也是（每一步按自己的命令键取自己最近一次跑成的用时，各片按片号各一份；计划那一步不交每片的项，各片按项估做不到）。`task status <号>` 在预计用时后面写这个数是按命令、按项还是没有记录。提交满两小时的排到最前、先判，不归交替管；放不下时不挡后面、不留份，后面放得下的照队列先后起。排着的一律不留账。GPU 任务照多机配置里卡的先后挑（判法在 `task_server.py` 文件头「GPU」）；卡的台标签写多机配置 `HOSTS` 里那一台的地址，每项 `<地址>:<卡号>:<额度>`（例 `192.0.2.1:0:32G`；地址可以是 IPv6，从右边切两刀），台标签不在 `HOSTS` 里的判配置写坏，服务照 `HOSTS` 认出自己这一台、交给命令的 `GPU_CARDS_BY_PRIORITY` 是原样那一项；带上卡特性（`gpu`、`tests-on-a-real-gpu`，连同 `<包>/<名字>` 与 `--all-features`）的 cargo 只有 `test`、`run`、`bench` 跑编出来的代码、又不带 `--no-run` 的才要卡，`clippy`、`check`、`build`、`doc` 与 `--no-run` 只编不跑，不要卡、不清场。按参数才要卡的脚本在自己文件头前 60 行写一行 `# task-server-gpu-arguments: <词> [<词>…]`：bash 或 python3 起它、第一个非选项参数是其中一个词就要卡（`*` 是任何参数都要卡），写法在 `task_server_lib.py` 文件头「GPU」。两台走同一段判法（记账、放不放得下、卡住与 CPU、峰值、上限、编译目录、挑台、盘余量，盘余量两台同一个 50G 下限），按台不同的只在怎么连上、怎么量、怎么起那一台（`task_server.py` 的 `Machine`）。尝试正常结束之后单元里还留着命令起的孙进程的，服务停掉、结局说明写明；跑超过 30 分钟而失败、脚本又没在文件头写 `# task-server-resumable: <词>…` 声明续跑的，结局说明写「这条跑了 N 分钟、没有续跑：重交会从头跑」。

每次尝试的命令外面套 `scripts/hosts/capped.sh <每台默认重活线程数>`，线程一律是这个数，提交时不收；要卡的任务一律在最外层再套 `scripts/crash/gpu-quiesce-run.sh --`。内存包装、`capped.sh`、清场包装与 `multi-host-run.sh --peer` 只认服务直接套的那一层：服务给每一层设一个准入标记（值是 `<任务号>.<第几次>`），那一层读掉、不交给命令，任务里面的脚本再套一层当场拒，出路是删掉那一层，内存、线程、卡照读服务交的 `TASK_SERVER_MEMORY_CAP_BYTES`、`SINGLEFS_THREAD_CAP`、`GPU_CARDS_BY_PRIORITY`；会编的 cargo 另给这一次一个仓外的 `CARGO_TARGET_DIR`，跑完留着，`wait` 末行带 `build=<目录>`，由提交它的会话 `task clean <号>` 删。定额度只照服务自己库里的尝试记录（近几次跑成的峰值的最大值 × 1.5；不读内存包装的峰值表；命令键先剥掉 `env` 与多机配置路径、`LANG` 这类不改跑法的变量前缀），没有记录的先记 4G（只判红过的不低于判红那一次的峰值 × 1.5），每条任务的上限与记账都不低于 4G（照记录算出来更小的也取 4G，钉值除外），撞了乘 1.5 重排，乘 1.5 会到这台总上限的不再重排，有总上限更大的另一台就换过去，没有就记 `memory-cap-at-maximum` 打回提交它的会话；算这台还放得下多少只算进程用的内存（`anon` 加 `shmem`），页缓存与内核挂在 slice 上的不算；起来之后先宽限 10 分钟不判；之后日志满 10 分钟没涨、这段时间 CPU 也几乎没涨（平均不到一个核的 1%）的服务停掉、记 `stalled`，读不到 CPU 不当没涨；跑过记录用时 5 倍、日志又满 10 分钟没涨的不看 CPU 也停掉、记 `stalled`（死循环也停），没有记录用时的不判这一条；`stalled` 不重试，打回提交它的会话；判卡住那一刻这一次的内存峰值已到上限 98% 的是顶在上限上卡住，照样停掉，但记 `memory-cap-hit`、乘 1.5 重排（钉值的打回 `memory-cap-at-maximum`），说明写「顶在上限上卡住」；已跑多久、记录用时、在用核数只写进 `task status`。第二台那一层的内存峰值照 `multi-host-run.sh` 打的 `MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<字节>` 记进那一次尝试，同本机一样进定额度，不打的记没量到。编译目录占盘也按命令记峰值（本机量服务给的编译目录，第二台照 `MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<字节>`），派到一台之前那台的盘余量减去在跑的还要涨的，要放得下这条命令的占盘峰值加 50G 下限，放不下就排着、排队原因点名「编译目录占盘」。命令自己退 75（prune-server 不可用）记 `prune-server-unavailable`、`wait` 退 75、不重排，服务另起一次 `python3 prune-server/control check`，不通写告警，不起、不停 prune-server。命令写成 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>` 的，算命令键、认登记表行照 `--` 后面的原命令（`--server-binary` 的路径不进键），命令键前面另接「〈task-server 临时 prune-server〉」：同一个 scope 里多起一份 prune-server，不与不包的那一条共用记录；包装照旧整条执行。

准入按上限记账：在跑的与新起的都按上限记（收到那台总上限减那台此刻的余量为止），每台在跑任务的上限之和不超过那台 slice 的总上限减那台此刻的余量，还没涨到量的那一截按「上限 − 此刻回收不掉的用量」算；两台同一段：用量由读那一台余量的那一次（第二台是同一次 ssh）按每条尝试的 scope 量回来（scope 照里面进程环境的 `TASK_SERVER_ATTEMPT_LABEL` 对到尝试上，两台都给），报告里没有那一条的 scope 记整份上限、不记 0，第二台那一层内存包装不另记账。余量是那台此刻 `singlefs-heavy.slice` 里回收不掉、又不归任何在跑任务的那一截（slice 自己记着、哪个子 cgroup 里都没有的，量法在 `scripts/hosts/memory_peaks_lib.py` 文件头「余量」），每次读两台的余量时各量各的，按 1 GiB 向上取整（量到 1.01G、1.1G 记 2G，2.1G 记 3G，0.7G 记 1G，恰好整数的不进位）；它不是常量，也不在配置里。哪一台量不到余量，那一台这一轮不派，排队原因写哪一台与原因；能派的每一台都量不到的不起、记 `run-condition-unmet`（`wait` 退 78），不退回固定的数。每次尝试经环境 `TASK_SERVER_MEMORY_CAP_BYTES` 把这一次的上限（字节，十进制）交给命令，本机与第二台都给。`python3 task-server/control pin-cap <命令…>` 给一条命令钉上限（`unpin-cap <命令…>` 撤；经服务写库，服务在跑时也能用）：钉了的命令上限与记账都取多机配置 `SLICE_MEMORY_CAP_BY_HOST` 里派到的那一台那一项减那台此刻的余量（`<地址>=<值>` 空白分隔，写的是那台 slice 的总上限，配多少就是多少；每次现读，不记字面的数；那一项不比那台的余量大的，`pin-cap` 当场拒、出路是把那一项改大），学到的峰值不顶掉它，撞了不放大、记 `memory-cap-at-maximum`，`task status` 写明是钉值；钉了而配置的 `SLICE_MEMORY_CAP_BY_HOST` 读不到（没有、写法不对、少一台、多了不在 `HOSTS` 里的地址、文件读不了）的不起、记 `run-condition-unmet`（`wait` 退 78），`wait` 的理由行写读不到的原因与出路（改对配置或 `unpin-cap`）。疑似卡住只通知：宽限过后整个单元的 CPU 满 10 分钟平均不到一个核的 1%（要卡的另要这段时间它那张卡上没有它的计算负载，量不到当有）的，不管日志涨不涨，服务不停它：`task status` 写「疑似卡住」与依据、记一条告警，`wait` 打一行 `TASK_SERVER_SUSPECT task=<号> …`、退 231，任务照跑，再起 `wait` 接着等（同一段只报一次，CPU 又涨了算这一段结束）。点到 `.claude/gate.d/` 的门禁派得到第二台（它跑的就是这棵共享的树）；带判定库而登记表 `.claude/gate.d/verdict-library-scripts.tsv` 那一行第三列还没写 prune-server 的，提交时就拒（下面「## 提交写法」）。`python3 scripts/experiment-apparatus/device-code-key.py run … --feature <值>` 的特性值带 gpu 的也要卡。`/health` 只做库写入试探与排队线程心跳，排头放得下却不起由排队线程每一轮算好、`/health` 只读。

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
- 挂载：第二台上每段 `stat -f -c %T -- <根>/.` 读到 `nfs`（走进去触发自动挂载）；
- 读写：两个方向各写一个探针文件，对方读得到（本机写、第二台读，第二台写、本机读）。

`task-server/control check`、`/health` 的 `nfs` 栏、`task-server/control watch` 同样判这四项：服务起来之后某一项转为不过，也在这三处看得到。

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
task-server/task submit -- <命令…>                 # 打出任务号；环境里的 SINGLEFS_RUN_SCOPE 这类白名单变量照带
task-server/task wait <号>                         # 用 run_in_background 起，完成时退出、末行 TASK_SERVER_RESULT …，退出码是任务的
task-server/task status [<号>]                      # 这个会话在排、在跑、完成了没人取的；排着的带「在等：…」，在跑的带已跑多久、日志多久没涨
task-server/task cancel <号>                        # 只许提交它的会话
task-server/task log <号>                           # 最后一次尝试的日志（最后 200 行；--all 整份；--attempt <第几次> 取那一次；任务或那一次不在退 4）
task-server/task clean <号>                         # 删这条任务留下的编译目录与现场（只许提交它的会话、有了结局才删；没设 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的编译目录有结局就删了、现场在 wait 取走之后一小时删；第二台那一趟的编译目录 multi-host-run.sh 自己删）
```

`wait` 自己报任务要处置的五种情形（用户在弹窗里选「task wait 自己报」）：一是排队过久，排满 `QUEUE_ATTENTION_MINUTES`（30 分钟）；二是跑到记录用时的 `OVERRUN_ATTENTION_FACTOR`（3）倍，日志在涨时服务不停它，靠这一条叫人；三是换卡重排，清场判出卡坏、服务换别的卡重排，结局可能是 `succeeded`；四是清场没收尾（`cleanup-pending`），命令已退出而清场回读没收干净，服务保留资源占用；五是分片的父任务推不动（`shard-advance-failed`）。另有第六种只报有结局的任务：改主工作区的命令（`apply-writer-patch.py`、`hook-swap.py`、`stage-mine.py`、不带 `--skip-apply` 的 `merge-and-verify.sh`）以撞上限、被挤掉、显存不够、第二台没跑成、撞顶、取消、卡住被停、对不回来或被信号杀收尾（`workspace-write-interrupted`），可能写了一半，`next=` 点名要核的文件或补丁目录（用户：「这个问题很严重 因为可能会导致补丁没有正常写入」）；`apply-writer-patch.py` 有结局文件 `apply-outcome.tsv` 的，服务照它的 `--read-outcome` 判，打全了、已退回的不报，带 `--dry-run` 的不报。服务把这条任务（分片的连同子任务）的这几样记成「要处置」的提醒，正在等它的 `task wait <号>` 读到一条新提醒就打一行 `TASK_SERVER_ATTENTION task=<号> type=needs-attention kind=<种类> tier=<档> basis=<依据的数> why=<原因> next=<出路>`，然后退 230：任务照旧在排或在跑，服务没动它，按 `next=` 处置，再起 `wait` 接着等。任务已有结局的（换卡重排跑成）这一行打在 `TASK_SERVER_REASON` 之前，退码仍是任务的。同一条任务同一种提醒、同一档只叫醒一次，重起 `wait` 不再叫，档跨过下一档（阈值的两倍、四倍……）才再叫；提醒记在服务内存里，服务重启后再报一次。服务健康不归 `wait`：主 agent 用 `run_in_background` 起 `python3 task-server/control watch` 盯着，它退出即处置。

命令里不套包装、不写资源参数：`run-with-memory-cap.sh <上限>`、`host-dispatch.py … --`、`multi-host-run.sh … --`、`nice`、`timeout`、`capped.sh`、`two-logs.sh`、`gpu-quiesce-run.sh` 与脚本自己的 `--memory`、`--memory-cap`、`--threads`、`--peer`、`--local-only`、`SINGLEFS_THREAD_CAP`：这几样由服务做，写了当场拒并点名去掉哪一样。白名单外的环境变量当场拒并点名；命令真要它的写成 `task-server/task submit -- env <名>=<值> <命令…>`。退役的变量（`SINGLEFS_HEAVY_TESTS`、`GATE_MUTATION_START_OVER`）设在环境里、写在命令里（`env` 后面与命令前面的 `名=值`）都当场拒，拒绝说明点名它、写出路；名单是 `task_server_lib.py` 的 `RETIRED_VARIABLES`。

派不到第二台的三种提交时当场拒（用户：「改成提交时当场拒（推荐）：不再悄悄改派本机：碰到这三种就在提交时报错、点名是哪个路径或哪一族、出路写怎么改，任务不跑。」）：`task submit` 退 3，打那一句与出路，任务不进队、不跑。一是工作目录与命令里的绝对路径（整个词、`名字=绝对路径` 的值、`bash -c` 这类一个词里的每一段）有不在多机配置 `NFS_SHARED_ROOTS` 任何一段下的，点名那个路径，出路是改放某一段下；`/dev/null`、`/usr/`、`/bin/` 这类两台相同的系统路径不算；`/tmp` 下的同样拒，那一句另说重启会清 `/tmp`。二是交给命令的 `SINGLEFS_DATA_ROOT`、`SINGLEFS_WORK_ROOT` 有不在共享根下的，点名变量与值，出路是配多机配置。三是命令点到登记表 `.claude/gate.d/verdict-library-scripts.tsv` 里带判定库的一行、而那一行第三列还没写 `prune-server` 的，点名那一族，出路是那一族接进 prune-server、把第三列翻成 `prune-server`；登记表读不了、写坏了的同样拒。

临时 prune-server 的包装（`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>`）只给脚本自证用（用户：「旧脚本不兼容 我再说一次 如果错了让 对方重新提交」）：剥出的原命令里有会编的 cargo、或要卡的（认法与服务定编译目录、分卡的是同一套），提交时照上面同一种形态当场拒：`task submit` 退 3，打那一句（点名会编还是要卡）与出路（去掉包装重交），任务不进队、不跑。剥开只用来判，收下的照旧整条执行；编译目录、分卡与清场只照外层那一条给，服务不剥包装去迁就。不带 `--server-binary`（命令前的赋值与任务环境里也没给 `PRUNE_SERVER_BINARY`）的包装自己要在服务给的编译目录里现编 prune-server，服务把外层这一条按会编的算：派本机、派第二台都给编译目录；带了 `--server-binary` 的不编、不给。checker 档的 `cargo test` 对真服务判、不套这层包装。

## 源文件头的声明

服务按源文件头前 60 行里的几行声明认一条命令要什么，调用方照旧不写卡、不写额度。声明由管那份入口或那个包的会话加，服务只读：

| 写在哪 | 写法 | 服务怎么用 |
|---|---|---|
| 入口脚本 | `# task-server-gpu-arguments: <参数>…`（`*` 是任何参数） | 第一个非选项参数是其中一个就要卡、由服务套清场 |
| 入口脚本 | `# task-server-gpu-cards: <张数>` | 要卡时要同一台的这么多张（下面「要几张卡」） |
| `.rs` 源（编出来的二进制的同名源；`cargo test` / `run` / `bench` 用 `-p` 点名的包里的源） | `// task-server-gpu-cards: <张数>` | 同上 |
| 入口脚本 | `# task-server-cargo-family: <cargo 参数>…`（例 `-p singlefs-verification --release --features gpu`） | 它里面起的 cargo 编哪个包、哪一档、哪几个特性：这条命令同键没有记录时，照同一编译族跑成过的峰值起步（下面「峰值起步」） |
| 入口脚本 | `# task-server-shards: <参数>…`、`# task-server-resumable: <参数>…` | 分片、续跑（`task_server_lib.py` 文件头「分片」「续跑」） |

要几张卡：用户定的真卡测法是「你需要测试的是 一张卡1gb 2张卡 三张卡 每张卡都能用来分片gc。没有卡报错。 有卡但是失败 报错加cpu兜底」。声明了张数的，服务照多机配置里卡的先后，挑头一台空卡凑得出这么多张、内存又放得下的，把那台空卡里排在前面的这么多张整组分给它：`GPU_CARDS_BY_PRIORITY` 交这几项用空格接的一串，清场整组清，几张在跑时都算占着。多机配置点名的卡哪台都凑不出这么多张的，结局记 `never-fits`（退 241）；张数写的不是一个正整数的，记 `run-condition-unmet`、说明点名哪一份写了什么。`.rs` 源读哪几份：点了 `--test <名>` 的读 `tests/<名>.rs`，只点 `--lib` 的读 `src/lib.rs`，别的读 `src/lib.rs`、`src/main.rs`、`src/bin/` 与 `tests/` 下的，几份都写了取最大，都没写是一张。例：`crates/singlefs-background-gpu/tests/<目标>.rs` 头几行写 `// task-server-gpu-cards: 3`，`cargo test --release -p singlefs-background-gpu --features tests-on-a-real-gpu --tests` 就分同一台三张。

峰值起步（服务自己库里的尝试记录，`task_server_lib.py` 文件头「资源记录」）：
- 编译族：会编的 cargo 按包、档（`--release`）、特性那几样归族，声明了 `task-server-cargo-family` 的入口照声明归族。同键一次跑完的都没有时，从同族最近 5 次跑成量到的最大峰值 × 1.5 起步，比 4G 大才换；有自己记录的照自己的。
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
- `control watch` 暂停着照样盯服务通不通、库写不写得进、单元在不在、NFS、第二台。
- 只许本机起服务的那个用户发 `pause`、`resume`，认法同提交、取消：服务看调用方的 uid 是不是它自己的。
- 空档的活做完一定要 `resume`；暂停着的服务平时看不出异常（`check` 退 0），忘了放开排着的任务会一直等。

## 自证

`python3 task-server/task_server_lib.py --selftest`：每格一份真的 uvicorn 挂在临时 socket 上，内存包装、第二台、清场、`control` 用的 systemctl 换成假脚本，尝试的 scope 换成假目录，不起编译、不连第二台、不上卡。它跑的全是模拟世界，主 agent 直接起、不经服务排队（钩子名单 `.claude/hooks/lib_project_code_runs.py` 的 `MAIN_AGENT_TOOLS_RUN_DIRECTLY`）；`TASK_SERVER_SELFTEST_CELLS=<格,…>` 只跑点名的格。弄坏开关 `TASK_SERVER_BREAK=<项>` 的表在 `task_server_lib.py` 文件头。
