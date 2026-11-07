# task-server 代码审核

审的是工作区里 `task-server/` 相对 HEAD `ff5dca7d2` 的未提交改动（已暂存与未暂存合在一起）。工作区里其他目录的改动没有纳入。这一轮没有改产品代码，没有跑自证、门禁或任务服务。

## 范围

| 文件 | 这一轮 |
|---|---|
| `task-server/task_server.py` | 取消产物、卡住提醒、看门狗收进服务的新增路径 |
| `task-server/task_server_lib.py` | 对照新增判法的文件头 |
| `task-server/task_server_selftest.py` | 对照取消产物与 quiet-log-busy 的自证格有没有盖住下面两条 |
| `task-server/task` | `cancel` 见到 `artifacts.left` 的退出码 |
| `task-server/control` | 未单独立项 |
| `task-server/README.md`、`README.en.md`、`README.ja.md` | 只在提醒文案和代码不一致时引用 |
| `scripts/hosts/multi-host-run.sh` | ⑥ 那一行何时才打、`peer_run_pending` 何时置上 |
| `task-server/cleanup-protocol.md` | 回执 `not-started` / `pending` / `confirmed` 的口径 |

diff 是 8 个文件，增加 3139 行、删去 548 行。行号是工作区当前文件的行号。

## 结论

确认 4 项。不是只有头两条。第二遍又对过收件箱推送、卡住判法、插空、全仓编译拒绝和客户端文件头。

| 编号 | 严重性 | 问题 | 状态 |
|---|---|---|---|
| TS-C1 | bug | 第二台尝试的清理回执仍是 `not-started` 时取消，被当成本地目录没删掉，`task cancel` 退 5 | 已改 |
| TS-C2 | suggestion | `quiet-log-busy` 与 `io-wait` 的说明在 CPU 并不符合文案时也会报 | 已改 |
| TS-C3 | bug | `push_outcomes` 把待发结局整批取出后，循环里抛错就丢掉还没发的任务号，下一轮不补发 | 已改 |
| TS-C4 | suggestion | 客户端文件头仍写提醒只有六种，服务和 README 已经是九种 | 已改 |

这批其余读过的部分：看门狗收进服务（收件箱推送结局、提醒与健康），卡住判法收紧（D 状态与 GPU 计算负载免停，贴着上限仍按撞上限重排），取消时删编译目录、现场和第二台这一趟的本地目录。内存起步（编译族、改工作区命令的首趟余量）和整片/碎片插空的账与文件头一致。提交时拒绝不点名包的全仓编译接到了 `placement_refusal`。这些没有另立项。

## TS-C1：回执还是 not-started 的取消被判没删干净

**位置：** `task-server/task_server.py:4091`（`cancelled_attempt_artifacts`），`3752`（`confirm_peer_cleanup`），`6013`（`start_attempt` 写回执），`task-server/task:404`（`cancel`）。退出码常量在 `task-server/task:69`，`CANCEL_UNCONFIRMED_EXIT = 5`。

`start_attempt` 在派到第二台时，起包装之前把 `peer-cleanup.json` 写成 `{"status": "not-started", "label": ...}`。`confirm_peer_cleanup` 对 `not-started` 和 `confirmed` 同一口径：清理已确认，不跑 `--recover-cleanup`，也不往日志补 ⑥。

取消产物走另一条口径。尝试的主机是第二台，且取消开始时这次尝试还在 `starting` / `running`（`before["active"]`），就只在日志最后一段里找同时含 `PEER_TRIP_REMOVED_MARK`（「⑥ 删了第二台这一趟的本地目录」）和 `PEER_TRIP_READ_BACK_MARK`（「回读不在了」）的一行。没有就 `gone` 为假，放进 `left`。`task cancel` 见到 `artifacts.left` 就退 5，并让人到第二台核 `builds/<标签>`。

包装要到 `peer_run_pending` 之后才打 ⑥。`scripts/hosts/multi-host-run.sh` 在建好 `builds/<标签>/` 之后才把 pending 置上；在那之前退出不删、不打 ⑥。因此在工具链比对、NFS 或 ssh 检查、建目录之前取消时：单元能停，回执仍是 `not-started`，第二台本地目录并未创建，取消仍退 5。

`cell_cancelled_artifacts_removed`（约 `task_server_selftest.py:8588`）要等假第二台目录建出来才取消。假 `multi-host-run.sh` 的退出路径只要有标签就打 ⑥，盖不住「回执停在 `not-started`、目录还没建」这一段。

**改法：** 产物核对先读这一次的 `peer_cleanup_file`。状态仍是 `not-started` 时不要报「没删成」，与 `confirm_peer_cleanup` 同一口径。只有回执到过 `pending`，或包装已经打过起跑行而日志里仍没有 ⑥「回读不在了」时，才列入 `left` 并让 `cancel` 退 5。自证补一格：第二台尝试停在记了 `not-started`、目录还没建出来时取消，应退 0，且不出现「没删掉…本地目录」。

## TS-C2：quiet-log-busy 与 io-wait 的说明宽过判据

**位置：** `task-server/task_server.py:3049`（`quiet_log_items`），`3068`（`io_wait_items`）。

`quiet_log_items` 只看 `log_growth` 里上一次增长时刻，日志安静满 `quiet_attention_seconds` 就报 `quiet-log-busy`。它不看这一窗口的 CPU，也不排除已经按 D 状态或 GPU 计算负载免停的尝试。`why` 写的是「日志不涨而 CPU 在涨（或读不到 CPU）」。三份 README 把这种提醒限定在 CPU 在涨或读不到。任务因此能在只因 D 状态或 GPU 负载才没被停时，同时收到 `quiet-log-busy`。`cell_attention_quiet_busy_log` 只用一直在涨的假 CPU，看不出和 `io-wait` / `gpu-busy-quiet` 叠在一起时文案是错的。

`io_wait_items` 只看 `io_waits` 的 `blocked_since`。CPU 在涨时，只要日志安静且每一轮都有进程在 D 状态，也会报 `io-wait`。它的 `why` 却写「日志与 CPU 都不涨」。

**改法：** `quiet-log-busy` 仅在服务因 CPU 在涨或读不到 CPU 而没有停时才报。已经按 `io-wait` 或 `gpu-busy-quiet` 免停的不要再报这一条。`io-wait` 的 `why` 改成与判据一致，不要写 CPU 也没涨。

## TS-C3：发结局的循环抛错之后，没发出去的不再补发

**位置：** `task-server/task_server.py:3302`（`push_outcomes`），`3256`（`inbox_loop`），`2530`（`notify_finished`）。

`notify_finished` 只把任务号放进内存里的 `inbox_pending`。`push_outcomes` 进循环之前把这整个集合取出来、换成空集。循环里 `summary`、`push_attention` 或发消息抛错时，异常回到 `inbox_loop`。那里写「下一轮再判」，但任务号已经不在 `inbox_pending` 里，也不会因为抛错写 `inbox_failed`。这一批里还没轮到的结局都不再发。

库写不进时这一路正好会抛。服务还在跑，结局仍在库里，`wait` 取得到。文件头 ⑥ 写的是「发不出去不重试」和「重启前记下了结局而还没发的不补发」（约 454 行）。那两句说的是发失败和重启，不是这一轮循环中途抛错。按 `inbox_loop` 自己的「下一轮再判」，抛错不该把还没发的号丢掉。

交下一条任务时，满 30 分钟没人取才会在提交警告里出现。在那之前，靠收件消息叫醒的会话看不到这次结局。

**改法：** 取出之后用一份还没发完的名单。某一条抛错或没发出去，把还没处理的号放回 `inbox_pending`，下一轮再发。已经发出去并记了已取的不要放回。

## TS-C4：客户端文件头的提醒种类少了三种

**位置：** `task-server/task:28`。对照 `task-server/task_server.py:3112`（`attention_of_record`）和 `task-server/README.md`「提交写法」里列的九种。

客户端文件头写「排队过久、跑到记录用时的很多倍、换卡重排、清场没收尾、分片父任务推不动、改主工作区的命令没正常结束，六种之一」。服务实际还会报 `quiet-log-busy`、`io-wait`、`gpu-busy-quiet`。README 把这三样算进八种，另加有结局才报的 `workspace-write-interrupted`，一共九种。`wait` 收到这三样仍退 230，但照文件头会以为只有那六种。

**改法：** 文件头改成与 README 同一份九种名单，点名这三样的 `kind`。

## 第二遍对过、没有另立项的

卡住停任务（`judge_stall_stop`）：贴着上限时 CPU 在涨也停；D 状态和卡上有计算进程的免停不罩贴着上限那一支。与文件头写的一致。

插空（`gap_fill`）：封组之后，放得下的是「总上限减余量、再减排头整片记账、再减挨着整片在跑的上限」。时长那一支不进 `beside_whole_capped`。与文件头一致。整片能不能起来仍过 `room_by_the_formula`。

不点名包的全仓编译（`whole_workspace_cargo_refusal`、`cargo_manifest_scope`）：带 `--workspace` / `--all` 的拒；没点 `-p` 且落到只有 `[workspace]`、没有 `[package]` 的清单也拒；包自己的清单放行。

收件令牌：`summary` 回话只留 socket（`task_server.py:6772`）。重启前没发出去的结局不补发，文件头 ⑥ 写了，不另算问题。

## 还没做

产品代码与自证都还没改。上面四条的状态保持「未改」。接着改的话，改完把落点和自证结果补在这一节后面。

### 改了（2026-11-06）

四条都改了，服务已重启换上，`task-server/control check` 退 0。

| 编号 | 落点 | 自证格 | 弄坏开关 | 复跑 |
|---|---|---|---|---|
| TS-C1 | `task_server.py` 新常量 `PEER_TRIP_BUILT_MARKS`（`multi-host-run.sh` 建好 `builds/<标签>/` 之后才打的两行）；`cancelled_attempt_artifacts` 日志里没有 ⑥「回读不在了」时先读 `peer_cleanup_file`，回执是 `not-started`、日志里也没有那两行的，不列进 `left` | `cancelled-before-peer-trip-built` | `cancel-not-started-receipt-unread` | 8988 只红这一格；`cancel-no-readback` 8993、`nfs-unchecked` 8992 也红在这一格，两条的点名都补了 |
| TS-C2 | `log_growth` 第五栏从布尔改成免停缘由（`SPARED_CPU_UNREAD` / `SPARED_CPU_BUSY` / `SPARED_IO_WAIT` / `SPARED_CARD_LOAD`）；`quiet_log_items` 只在缘由是 CPU 在涨或读不到 CPU 时报；`io_wait_items` 的 why 不再写 CPU；三语 README 第六种提醒那一句跟着改 | `io-wait-local-not-stalled`、`gpu-busy-quiet-not-stalled` 两格加了「不打 quiet-log-busy」的判断 | `quiet-log-busy-any-exemption` | 8989 红在那两格 |
| TS-C3 | `push_outcomes` 取出待发的号之后用 `unsent` 记还没处理完的；某一条抛错时，这一条和它后面的号在 `finally` 里放回 `inbox_pending`；发出去并记了已取的、发不出去写了告警的不放回 | `inbox-outcomes-resent-after-error` | `inbox-pending-dropped-on-error` | 8990 只红这一格；`pushed-outcome-not-collected` 8991 红在这一格与原点名的 `inbox-outcome-succeeded` |
| TS-C4 | `task-server/task` 文件头「六种之一」改成「九种之一」，逐个写名字与 kind（含 `quiet-log-busy`、`io-wait`、`gpu-busy-quiet`），与 `task-server/README.md` 九种逐个对上 | 无 | 无 | 报告里逐个对照 |

新格与邻格 7 格 8987 绿；整份自证 8994「270 格都对」；开关表与 `KNOWN_BREAK_ITEMS` 都是 305 条。

没盖住、另立：`multi-host-run.sh` 建好目录到打起跑行之间被 SIGKILL 时，回执仍是 `not-started` 而目录已建，这一版当成还没建。根子是第二台停命令时退出回执写不出来，归下一批改停的那一方（记录 `records/2026-10-31-task-server巡检.md` 的批 91）。
