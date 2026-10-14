# scripts/hosts

内存与线程上限、双机派发与资源测量、长活脱离会话跑。两台跑同一份，差异只走参数与仓根的 `multi-host.env`（不进 git，模板 `multi-host.env.example`）。

| 文件 | 做什么 |
|---|---|
| `run-with-memory-cap.sh`、`memory_peaks_lib.py` | 给一条命令定内存上限、放不下就排队；实测峰值记在 `~/.local/share/singlefs/memory-peaks.tsv` |
| `capped.sh` | 给一条命令定线程上限；上限写 `default` 取每台默认重活线程数 |
| `heavy-threads.sh`、`heavy_threads.py` | 每台默认重活线程数的两份取法（shell 的 `heavy_thread_default`、python 的 `default_heavy_threads`）：`SINGLEFS_THREAD_CAP` 设了照它，否则读 `.claude/heavy-thread-default` 的登记值，写坏或不在就拒 |
| `multi-host-run.sh`、`peer-host-lib.sh`、`host-dispatch.py`、`host-resources.py` | 把活挪到第二台跑、两台共用的那几段、按资源派发、量各台此刻能用多少 |
| `detached-run.sh` | 超过后台时限的长活挂到用户级 systemd 下跑 |
| `compile-then-swap.py`、`source-stamp.py`、`target-prune.py`、`target-prune.conf` | 先编后换、换了源码没重编的拦法、清编译目录的旧产物 |
| `minimal-copy.py` | 建最小仓副本：只拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo` 与 `.claude/heavy-thread-default`（`--with-gate-tables` 再加 `.claude/gate.d`），拷前报大小、超阈值拒绝，拷过去的修改时刻是此刻 |
| `work-directory-ownership.sh` | 工作根下每个目录的归属文件（会话号、建于、用途）：`claim` 写、`show` 读、`sweep [--dry-run]` 只删归属是本会话又没有进程开着的，别的会话的与没有归属文件的逐项列出交用户，删前逐项连大小写进 `<工作根>/sweep-<日期>.log` |
| `cache-keepalive.sh`、`test-environment-check.py` | 子 agent 等长活时续提示缓存的计时器（agent 定义与共用约束都不再叫子 agent 起它）；测试环境的残留检查 |
