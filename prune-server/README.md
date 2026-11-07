# prune-server

singlefs 的判定库服务：RocksDB 判定库的唯一持有者，两台的任务都经它读写同一份库。用户级 systemd 服务 `singlefs-prune-server.service`，本机走 unix socket（`${XDG_RUNTIME_DIR}/singlefs-prune-server/server.sock`），第二台走 TCP（地址取仓根 `multi-host.env` 的 `PRUNE_SERVER_TCP_ADDRESS`）。协议在 `src/protocol.rs` 文件头，容错客户端在 `src/reliable_client.rs`。

| 文件 | 做什么 |
|---|---|
| `control` | 装、起、停、查、监听：`install [--dry-run] [--binary <编好的 prune-server>]`、`start`、`stop`、`check`、`watch`、`drop-oversized-values [--dry-run]`；派到哪台都只管服务那台的单元（不在服务那台就经 ssh 转去）；用法、退出码与自证在文件头 |
| `singlefs-prune-server.service.in` | 用户级单元模板，`control install` 照它填路径；`Restart=always`、`RestartSec` 在模板文件头 |
| `src/bin/prune_server.rs` | 服务（`serve`）、控制命令调的 `admin`、自证；用法、自证的格、弄坏开关表在文件头 |
| `src/storage.rs` | 库的布局（只有默认列族、键原样、记账行的键前缀）、开库选项、开库之前抬文件数软上限与核够不够、同步 WAL 失败之后退出重开库的规则 |
| `src/server.rs` | 准入：待处理帧数与字节数的额度（收着的帧体、待处理的帧、没写出去的回应都记在里面）、帧读取期限、写回应期限、连接数上限 |
| `src/journal.rs` | 容错客户端的追加日志：格式（含请求号高水位）、整理、旁边的 `<日志>.lock` 排他锁 |
| `src/fault_log.rs` | 故障行的写法 |
| `src/oversized_values.rs` | 离线子命令 `drop-oversized-values`：服务停着时删掉一把键单独一帧也读不回的整值（「库」一节） |

## 装

1. 编（经统一任务服务，不在本机直接起 cargo）。编译目录归服务、派到哪台都可能、跑完就删，所以同一条命令里把二进制拷到多机配置 `NFS_SHARED_ROOTS` 某一段下的目录（两台同一绝对路径读得到）：
   `task-server/task submit --cwd <仓根>/prune-server -- bash -c 'cargo build --release --offline --bin prune-server && cp "${CARGO_TARGET_DIR:?}/release/prune-server" <共享根下的目录>/prune-server.new && mv <共享根下的目录>/prune-server.new <共享根下的目录>/prune-server'`，再 `task-server/task wait <号>`；`<共享根下的目录>/prune-server` 是二进制。
2. `python3 prune-server/control install --dry-run --binary <那个二进制>` 看每一步。
3. `python3 prune-server/control install --binary <那个二进制>`：拷成 `~/.local/share/singlefs/prune-server/bin/prune-server`（回读 sha256），建状态目录 `~/.local/state/singlefs/prune-server`（700），写单元、`daemon-reload`、`start`，回读服务通了才退 0。不 `enable`，单元也没有 `[Install]` 段：开机不自起（用户定：「开机不自启动。 但是启动后要有恢复能力。」），起来之后才靠 `Restart=always` 恢复。单元带 `Restart=always`：进程不是 `systemctl stop` 停掉的（被 earlyoom 杀、崩了、退了），隔 `RestartSec` 秒由 systemd 拉起，落在容错客户端一轮重连（至多 120 秒）里，客户端重连上接着做。服务的库是状态目录下的 `library`，服务头一次起时建。
4. 用完 `task-server/task clean <号>` 删编译目录。
5. 换二进制：再跑 3（单元在跑的不重起），然后 `control stop`、`control start`。

## 库

服务不原地打开别的库，只开自己那一份（状态目录下的 `library`）：只有默认列族，三张表都落在它里面、键原样，去重与认领两张记账表也在里面、各一个 0xF0 起的键前缀（`src/storage.rs` 文件头）。

状态目录下已有的库不是这一种（另有列族，例如早先多列族布局建的）时服务拒起退 6：挪开那个目录让服务建一份新的，判定在新库里重判。

### 一把键单独一帧也读不回的整值

大值分段（`scripts/prune-server/prune_server_client.py` 文件头「键空间」）之前写进来的整值，有的单个就长过成批点读一帧回得了的长度：服务整帧拒、客户端拆到一把键还被拒，那把键永远读不出。读路径上不兼容这种行，服务停着时离线删掉，删掉的判定由各入口下次经服务现判、分段写回：

派到哪台都一样：`control drop-oversized-values` 判服务在哪台，不在这台就经 ssh 转去；单元在跑就退 78；二进制与状态目录取装好的单元文件 ExecStart，起 `<二进制> drop-oversized-values --state-directory <状态目录> [--dry-run]`，输出原样打、退它的退出码（`control` 文件头）。

1. `python3 prune-server/control stop`。
2. `python3 prune-server/control drop-oversized-values --dry-run`，只打清单（每行 `PRUNE_SERVER_OVERSIZED_VALUE table=… value_bytes=… key=…`，末行 `PRUNE_SERVER_DROP_OVERSIZED_VALUES mode=dry-run … oversized_rows=N`），库不动。
3. 同一条去掉 `--dry-run`：删掉清单里的行、同步 WAL、逐把点读核没了，末行 `mode=dropped`，退 0。
4. 再带 `--dry-run` 跑一次，末行 `oversized_rows=0`。
5. `python3 prune-server/control start`、`python3 prune-server/control check`。

认哪几行：键不是段键、版本登记键的形状，值长让只读这一把键的成批点读回应超过帧长上限（与服务整帧拒同一份判法，`src/oversized_values.rs` 文件头）；判定向量两张表的空值行认不上。服务开着库时退 3（RocksDB 的 LOCK），退出码全表在 `src/bin/prune_server.rs` 文件头。

## 控制命令

派到哪台都只管服务那台的单元。每个子命令先判服务在不在这台：多机配置 `PRUNE_SERVER_TCP_ADDRESS` 的 IP 与多机配置的共用取法（`scripts/hosts/multi_host_addresses.py` 的 `resolve`：整份配置判过才给主机数组 `HOSTS` 里这一台与第二台的地址）给的两个地址比，是这一台那一项就在这台做；是第二台那一项，就经 ssh 直连那个 IP（用户名照这台的 `~/.ssh/config`），在那台同一绝对路径的仓里起同一条 `python3 <仓根>/prune-server/control <同样的参数>`，对端的标准输出、标准错误原样打出来、退它的退出码。

- 不过就退 78 的三样：多机配置里有 `PRUNE_SERVER_TCP_ADDRESS`（写 `<IP>:<端口>`，不写主机名）；共用取法取得到地址（配置整份判得过：`HOSTS` 写了、那个 IP 在 `HOSTS` 里、别的键齐，原因照抄判法那一句）；经 ssh 之前，这一条不是别的那台转来的（两台都不认那个 IP 是自己的，就是配置错了）。
- `install --binary` 转去时，二进制要在多机配置 `NFS_SHARED_ROOTS` 某一段下（两台同一绝对路径读得到），不在退 78。
- `watch` 转去时照样前台、按行转打，对端退出就退它的退出码；ssh 断了或连不上按「服务不通」报、退 1。别的子命令 ssh 连不上退 78。
- `control --selftest`：这份文件自己的自证（判在哪台、经 ssh 转、共用取法与本份的几种拒绝、watch 转打与断开、单元自己拉起与不 enable、stop 之后不被拉起、stop 读不到 cgroup 不当空、缺二进制时的编法提示、watch 认 activating 等过渡状态、drop-oversized-values 的拒与转起），用假的 ssh、假的 systemctl 与整份判得过的假配置（这一台写每台 lo 上都有的 127.0.0.1，第二台写文档段地址），共用取法照真的判法判，不起真服务；格与弄坏开关 `PRUNE_SERVER_CONTROL_BREAK` 的表在 `control` 文件头。

- `control check`：二进制、多机配置里的 TCP 地址、服务通不通，都过退 0，缺一样退 78；另打库目录（装好的单元 ExecStart 的 `--state-directory` 下的 `library`）。
- `control start`：起单元，回读 `is-active` 是 `active` 且服务通才退 0。
- `control stop`：立刻停，不等在跑的读写。先叫服务宣布停服务（服务尽力给连着的客户端各发一帧「服务停了」，收到的退 75 不再重试；限时 5 秒内没写进套接字的连接，服务往故障日志记一行 `stop-notice-undelivered`，那几条客户端收不到通知，只能靠重连上限退出，`control stop` 照样往下停、可以退 0），打出停之前连着的客户端（主机、任务号、第几次、进程号），排他写进状态目录 `stops/` 下一份带时刻的停服务记录，再 `systemctl --user stop`（这样停的 systemd 不拉起；不 disable，本来也没 enable：开机不自起）；回读 `is-active` 是 `inactive`/`failed`、单元 `cgroup.procs` 为空（`systemctl show` 问 cgroup 失败、`cgroup.procs` 读不了都不算空，退 1），隔单元的 `RestartSec` 加一秒再回读 `is-active` 仍是 `inactive`/`failed`（没被拉起）才退 0。最后对每个任务号打一行 `task-server/task status <号>`。
- `control watch`：前台循环（主 agent 用 run_in_background 起）。单元不在跑（`is-active` 打的不是 `active`、`activating`、`reloading`、`deactivating` 之一）、服务不通、故障日志里有这次 watch 起之后新增的故障行，判出一样就打出来退 1：出了什么、故障行原文、此刻连着的客户端（拿不到写拿不到）、每个任务号的 `task-server/task status <号>` 查法。一直好就一直查。
- `control drop-oversized-values [--dry-run]`：单元在跑退 78（出路先 `control stop`）；停着时起装好的单元 ExecStart 的二进制跑同名子命令，`--state-directory` 取单元的，退它的退出码（「库」一节）。

## 退出码

| 谁 | 码 | 意思 |
|---|---|---|
| 容错客户端（调用方照 `ClientFailure::exit_code` 退） | 75 | prune-server 不可用：服务有意停了（「服务停了」）、重连到上限（缺省 30 次、120 秒，「连不上」）、同一帧忙到上限。没确认的写帧留在追加日志里，重交时重放 |
| | 3 | 别的错（服务拒了帧、协议错、追加日志出错、调用方要停）；一帧大过帧长上限（`src/protocol.rs` 的 `MAXIMUM_FRAME_BODY_BYTES`）的当场报协议错、不记追加日志、不发、不重试，服务回「帧长度不对」的同样报协议错、不重连（`src/reliable_client.rs`、`src/client.rs` 文件头）；Python 客户端的大值分段在 `scripts/prune-server/prune_server_client.py` 文件头「键空间」 |
| `control` | 0 / 1 / 2 / 78 | 成了 / 回读不对或 watch 判出问题（含经 ssh 的 watch 断了）/ 用法错 / 运行条件不满足；经 ssh 转去的退服务那台的退出码；`drop-oversized-values` 起了二进制的退二进制的退出码（下面那一行） |
| `prune-server admin` | 0 / 1 / 2 / 75 | 成了 / 服务回了错 / 用法错 / 连不上服务 |
| `prune-server drop-oversized-values` | 0 / 1 / 2 / 3 / 6 / 8 / 78 | 成了 / 删完点读还读得到 / 用法错 / 库打不开（服务开着它、目录不在）或读写出错 / 库对不上 / 文件数上限不够 / 准入不满足 |
| `prune-server serve` | 1 / 2 / 3 / 4 / 5 / 6 / 7 / 8 / 78 | 监听停了 / 用法错（含认不得的参数）/ 库打不开（被别的进程持有、目录建不了）/ socket 不能用 / TCP 不能用 / 库对不上（目录里有文件却不是 RocksDB、另有列族、选项对不上）/ 同步 WAL 失败、服务退出（重起时从 WAL 重开库，`src/storage.rs` 文件头）/ 文件数上限不够（下面一段）/ 准入不满足 |

退 8（文件数上限不够）：RocksDB 开库时把库里的 SST 全打开，库有几个文件就占几个描述符。服务与自证起的临时服务每次开库之前先把 RLIMIT_NOFILE 的软上限抬到硬上限，再核够不够（库目录的文件数、进程此刻开着的描述符数与写死的余量之和，余量的算法在 `src/storage.rs` 文件头「文件数上限」）；抬完仍不够就不开库，标准错误写「文件数上限不够：要 N、硬上限 M」。出路是请有权限的人把起它的那一层的硬上限抬到至少 N（不超过 `/proc/sys/fs/nr_open`）再重起；单元模板里不另写软上限。

## 故障行

服务每出一个自身的错（读写库出错、分派错、线程 panic、宣布停服务时通知没送达）往 `~/.local/state/singlefs/prune-server/faults.log` 追加一行，客户端发错格式被拒的不记：

```
PRUNE_SERVER_FAULT time=<零时区 ISO 8601，到毫秒> kind=<storage-read-failed|storage-write-failed|dispatch-bug|internal-panic|stop-notice-undelivered> server_pid=<数> connection=<数> transport=<unix|tcp> peer=<地址> host=<主机名> task=<任务号> attempt=<第几次> pid=<客户端进程号> detail=<说明>
```

身份那几段来自客户端连上时的报到（`TASK_SERVER_TASK`、`TASK_SERVER_ATTEMPT`、主机名、进程号；不在 task-server 里跑的、不报到的写 `-`）。

## 第二台怎么连

第二台的客户端连 `multi-host.env` 里的 `PRUNE_SERVER_TCP_ADDRESS`（本机在有线直连段上的地址），与本机的服务读同一份配置。TCP 这一版不认证、帧不带校验列，只开在有线直连段的地址上。

## 出了问题怎么处置（主 agent）

1. `control watch` 退出，看它打的那几行：是不是服务自身的缺陷（故障行、服务不通、单元不在跑）。不是的（任务自己的红）交 task-server 照常打回。
   服务退了、被杀了，systemd 隔 `RestartSec` 秒自己拉起，客户端重连上接着做；watch 判出单元不在跑而拉起之后又反复退出的，才往下走。故障行写着「同步 WAL 失败之后服务退出」的，是服务退了 7：先查库所在的盘与文件系统（`dmesg`、`df`），盘好了往下走第 5 步重起（不用换二进制）。
2. 是：`python3 prune-server/control stop`，记下它打的客户端清单与停服务记录的路径。
3. 照清单里每个任务号 `task-server/task status <号>` 查是哪个会话交的，通知它们取消对应任务（客户端已退 75）。
4. 修。
5. 换二进制（见「装」第 5 条）、`python3 prune-server/control start`，`control check` 退 0。
6. 再起 `control watch`。
7. 通知那几个会话可以重交。
