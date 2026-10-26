# scripts/hosts

内存与线程上限、双机派发与资源测量、长活脱离会话跑。两台跑同一份脚本与同一份多机配置：起活那一台仓根的 `multi-host.env`（不进 git，模板 `multi-host.env.example`）是唯一的源，往第二台起活的入口每一趟原样带过去、核 sha256，第二台那条命令带 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer` 照角色换读（`peer-host-lib.sh` 文件头「两台用同一份多机配置」）；差异只走参数。

| 文件 | 做什么 |
|---|---|
| `run-with-memory-cap.sh`、`memory_peaks_lib.py` | 给一条命令定内存上限、放不下就排队；实测峰值记在 `~/.local/share/singlefs/memory-peaks.tsv` |
| `capped.sh` | 给一条命令定线程上限；上限写 `default` 取每台默认重活线程数 |
| `two-logs.sh`、`two_logs.py` | 日志出两份：`two-logs.sh [--quiet] <名> -- <命令…>` 照原样转打输出，同时写 `<名>.log`（全量）与 `<名>.problems.log`（只收错误与警告，行前带全量日志里的行号，末行写收了几行、全量几行），退出码是被包命令的；`--from-log <名>` 从已有的全量日志现筛。收哪些行：门禁的 `✗` 与它的 `→` 出路、`⚠`、本次未跑、逐格汇总，cargo / rustc 的 error 与 warning、libtest 的 FAILED 与 panicked、Traceback、非 0 退出码。入口脚本与门禁里的 `.log` 都经它写，一般只看 `.problems.log`；配套钩子 `.claude/hooks/two-logs-guard.sh` 拒绝手敲的 `> x.log`、`\| tee x.log` |
| `heavy-threads.sh`、`heavy_threads.py` | 每台默认重活线程数的两份取法（shell 的 `heavy_thread_default`、python 的 `default_heavy_threads`）：`SINGLEFS_THREAD_CAP` 设了照它，否则读 `.claude/heavy-thread-default` 的登记值，写坏或不在就拒 |
| `multi-host-run.sh`、`peer-host-lib.sh`、`host-dispatch.py`、`host-resources.py` | 把活挪到第二台跑、两台共用的那几段（含把多机配置带到第二台的 `peer_host_carry_configuration` 与第二台那条命令的角色 `peer_host_role_environment`）、按资源派发、量各台此刻能用多少 |
| `detached-run.sh` | 超过后台时限的长活挂到用户级 systemd 下跑 |
| `resource_release_broadcast.py` | 资源释放广播：内存包装、清场包装（`scripts/crash/gpu-quiesce-run.sh`）、`multi-host-run.sh --peer`、`detached-run.sh` 跑完各发一条事件（`emit`：写本机事件目录，按多机配置推给别的机器），内存包装排队与 `host-dispatch.py` 等待时收到就叫醒重判、不自动挪（`follow`、`wait`、`latest`） |
| `core_usage.py` | 长跑在用几个核：给一棵进程树取样、按 10 分钟窗口判在用核数与同时在算的线程（低于每台默认重活线程数的四分之一报）、读程序当场声明的串行段、写告警记录；`multi-host-run.sh --peer`、`detached-run.sh watch` 与 `scripts/agents/agent-watch.py` 共用 |
| `compile-then-swap.py`、`source-stamp.py`、`target-prune.py`、`target-prune.conf` | 先编后换、换了源码没重编的拦法、清编译目录的旧产物 |
| `minimal-copy.py` | 建最小仓副本：只拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo` 与 `.claude/heavy-thread-default`（`--with-gate-tables` 再加 `.claude/gate.d`），拷前报大小、超阈值拒绝，拷过去的修改时刻是此刻 |
| `work-directory-ownership.sh` | 工作根下每个目录的归属文件（会话号、建于、用途）：`claim` 写、`show` 读、`sweep [--dry-run]` 只删归属是本会话又没有进程开着的，别的会话的与没有归属文件的逐项列出交用户，删前逐项连大小写进 `<工作根>/sweep-<日期>.log` |
| `rocksdb-prebuilt.py` | librocksdb-sys 的 C++（`librocksdb.a`、`libsnappy.a`）每台主机只编一次，放 `~/.local/share/singlefs/rocksdb-prebuilt/` 按版本、rustc、C/C++ 编译器与指令集分目录；各编译目录经进程环境的 `ROCKSDB_LIB_DIR`、`ROCKSDB_STATIC`、`SNAPPY_LIB_DIR`、`SNAPPY_STATIC` 链它（不写进 cargo 配置：`[env]` 进变异的键）。`build` 持锁编一次、`install` 另写这台的三处环境来源、`check` 判这台、`audit-target <编译目录>` 判编译目录里现编过没有、同一把键下见过几种取值 |
| `cache-keepalive.sh`、`test-environment-check.py` | 子 agent 等长活时续提示缓存的计时器（agent 定义与共用约束都不再叫子 agent 起它）；测试环境的残留检查 |

审核回归测试：`python3 -B -m unittest discover -s scripts/hosts/tests -v`。使用临时目录检查清理的保留规则、备份隔离、Git 状态复查、会话路径边界，以及源码副本路径与等待时限的校验；各入口原有的 `--selftest` 另行运行。
