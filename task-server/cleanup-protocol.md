# 任务收尾与 GPU 阶段日志

收到命令退出码后，还必须确认进程清理完成（第二台那一趟、单元与 scope），才能把尝试记成 `ended`、归还资源。cgroup 读不到与 cgroup 为空是两种结果；清理后仍有进程或回读失败时，尝试保留原状态，并把原因写入 `cleanup_pending`。下一轮对账继续检查，服务重启后也从尝试记录恢复占用。服务不分卡，收尾不回收卡（卡归 GPU 批处理验证服务）。

新建的远端尝试在本机尝试目录保存 `peer-cleanup.json`。远端命令启动前原子写入 `pending`；SSH 失联、父进程消失但没有退出回执、目录持有者读不到，都不算完成。远端进程身份同时核 PID、启动时间和工作目录。已确认停止的进程树另记 `processes_stopped`，防止包装退出后重试误判，也避免按过时 PID 再次发送信号。删目录之前先停第二台这一趟内存包装的 scope（单元名在 `runs/<标签>.scope`，`multi-host-run.sh` 文件头 ⑥）；停了之后 scope 里还留着进程的不删、记 `pending`（原因 `scope-processes-remain`），留下的进程逐个记进回执的 `scope_leftovers`（服务把它们抄进 `cleanup_pending`，提醒照它换修改意见）。回读没有目录持有者后写 `confirmed`，再清目录。

包装退出后仍未确认的，由服务调用内部入口 `multi-host-run.sh --recover-cleanup <回执路径>` 重查。恢复连接可以继续收尾；开机标识改变则说明旧开机的进程已不存在。父进程和退出回执都丢失时保留占用，不猜测成功。不要删除或改写回执来放行。

## 读不到单元状态

对账读不到一次尝试的单元状态（`systemctl --user show` 起不来、超时或没打 `ActiveState`）时判不出单元在不在。留了 `result.json` 的照它的退出码收尾。没留的不杀、不收尾、不记结局，尝试照记 `running`，不设到点就收尾的时限，改用不经 systemd 的办法核这一次还有没有进程（只读 `/proc` 与 cgroupfs，不起 `systemctl`、不发信号）：

1. 本机读单元那一层（`attempt_runner` 与它起的包装都在本机的单元里，派到第二台的也是）：起跑标记 `started` 记的 runner 进程号在 `/proc` 里在不在；本机用户级管理器的 cgroup 树里名字是这个单元的目录、`attempt_runner` 写的 `scope` 文件点名的目录、名字以 `singlefs-memory-cap-` 起头而里面有进程的环境带这一次尝试标签的 scope，各自的 `cgroup.procs` 读出来空不空。
2. 派到第二台、清理回执 `pending` 的，第二台经多机配置的读法（`host-dispatch.py` 的 `Peer.run`）跑同一段（`task_server_lib.attempt_presence_readings`），读 `multi-host-run.sh` 记的 `runs/<标签>.pid` 与 `runs/<标签>.scope`；回执 `not-started`、`confirmed` 的，第二台没有这一次的进程。
3. 有一个进程号在、或有一个 cgroup 里有进程：还在，照记 `running`。cgroup 都读得到且空（目录已收掉而上一层列得出也算空）、记下的进程号都不在，连着满 `LOST_GRACE_SECONDS`（10 秒）：照没留结果收尾，记 `launcher-lost`，收尾时不再经 `systemctl` 回读单元里留下的进程，照这一回核的认。别的（cgroup 读不了、起跑标记不在、清理回执读不到、第二台连不上）是核不出：一直等。

读不到的那一段按档提醒发起任务的会话（要处置的提醒 `unit-unreadable`，头一档 60 秒，之后每翻一倍再推，同一档不重推），`/health` 的 `problem` 与 `unit_unreadable` 栏报本机读不到哪几次尝试的单元状态，`control check` 照它退 78。不要删尝试目录、起跑标记或改库来放行；修好本机的用户级 systemd 之后服务照常判。判法在 `task_server.py` 文件头 ①「读不到单元状态」。

日志字段：

- `MULTI_HOST_RUN_CLEANUP`：微秒时间、包装 PID、尝试标签、pending/confirmed 与原因。
- `TASK_CLEANUP` / `TASK_RESOURCES_RELEASED`：确认清理与写入结束状态是两条独立事件，带时间与尝试 ID。
- `GPU_LIFECYCLE`：Unix 微秒、进程内单调时钟、PID、线程、测试名，以及借卡后的卡序号和 PCI 地址。阶段覆盖判表过库、借卡、准备缓冲和管线、提交、等待、读回、释放缓冲及归还额度前的等待。`lease_return_handler_end` 仅表示处理函数返回，不能单独解释为额度已经释放。

GPU 日志直接写 stderr，绕过 Rust 测试框架的输出捕获，所以挂起或异常退出前已经进入的阶段也会留在标准错误里。没经过滤的运行里这些行照原样留在标准错误上；门禁 crash-point-replay 与变异重放（crates-mutation-replay）起的测试，行首是 `GPU_LIFECYCLE` 的行不进任务日志，只留条数和末尾现场：前者的条数在 `<日志>.bulk-lines.counts`，末尾 2 万到 4 万行在 `<日志>.bulk-lines` 与它的 `.previous`；后者在交回的标准错误末尾写一行汇总 `GPU_LIFECYCLE_DROPPED lines=<条数> bytes=<字节>`，这一格被停时才把最后 32 行打进任务日志。日志写入失败不会引发额外 panic。

离线回归入口为 `python3 task-server/tests/test_cleanup_protocol.py`，经正常任务服务提交。它模拟 SSH、systemd 和 cgroup，不连接另一台主机、不启动服务、不使用 GPU。读不到单元状态的自证是 `python3 task-server/task_server_lib.py --selftest` 里 `unit-unreadable-` 起头的四格：读单元状态那一步照自证世界里的文件演读不到，cgroup 读不到也照文件演，进程与本机的 cgroup 是真的。

这些改动不会自动重启或重载服务。调度器的新协议在以后加载新版 task-server 时生效；升级前已创建、没有回执路径的尝试不会被当作采用了新协议。GPU 日志在包含改动的新二进制运行时生效。
