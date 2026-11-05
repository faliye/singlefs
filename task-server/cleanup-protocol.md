# 任务收尾与 GPU 阶段日志

收到命令退出码后，还必须确认清理完成，才能把尝试记成 `ended`、归还显卡和资源。cgroup 读不到与 cgroup 为空是两种结果；清理后仍有进程或回读失败时，尝试保留原状态，并把原因写入 `cleanup_pending`。下一轮对账继续检查，服务重启后也从尝试记录恢复占用。

新建的远端尝试在本机尝试目录保存 `peer-cleanup.json`。远端命令启动前原子写入 `pending`；SSH 失联、父进程消失但没有退出回执、目录持有者读不到，都不算完成。远端进程身份同时核 PID、启动时间和工作目录。已确认停止的进程树另记 `processes_stopped`，防止包装退出后重试误判，也避免按过时 PID 再次发送信号。回读没有目录持有者后写 `confirmed`，再清目录。

包装退出后仍未确认的，由服务调用内部入口 `multi-host-run.sh --recover-cleanup <回执路径>` 重查。恢复连接可以继续收尾；开机标识改变则说明旧开机的进程已不存在。父进程和退出回执都丢失时保留占用，不猜测成功。不要删除或改写回执来放行。

日志字段：

- `MULTI_HOST_RUN_CLEANUP`：微秒时间、包装 PID、尝试标签、pending/confirmed 与原因。
- `TASK_CLEANUP` / `TASK_RESOURCES_RELEASED`：确认清理与写入结束状态是两条独立事件，带时间、尝试 ID 和显卡。
- `GPU_LIFECYCLE`：Unix 微秒、进程内单调时钟、PID、线程、测试名，以及借卡后的卡序号和 PCI 地址。阶段覆盖判表过库、借卡、准备缓冲和管线、提交、等待、读回、释放缓冲及归还额度前的等待。`lease_return_handler_end` 仅表示处理函数返回，不能单独解释为额度已经释放。

GPU 日志直接写 stderr，绕过 Rust 测试框架的输出捕获，所以挂起或异常退出前已经进入的阶段也会落到任务日志。日志写入失败不会引发额外 panic。

离线回归入口为 `python3 task-server/tests/test_cleanup_protocol.py`，经正常任务服务提交。它模拟 SSH、systemd 和 cgroup，不连接另一台主机、不启动服务、不使用 GPU。

这些改动不会自动重启或重载服务。调度器的新协议在以后加载新版 task-server 时生效；升级前已创建、没有回执路径的尝试不会被当作采用了新协议。GPU 日志在包含改动的新二进制运行时生效。
