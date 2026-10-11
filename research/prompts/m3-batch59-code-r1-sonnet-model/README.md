# m3-batch59-code-r1 正推腿模型与日志（2026-10-10）

原件在 `~/.local/share/singlefs/work/m3-batch59-code-r1-sonnet/`。脚本里的路径是那台机器上的绝对路径，拷到别处要改 `W=`、`FZ=`。

- `run.sh`：经 `capped.sh 15`、`run-with-memory-cap.sh 10G`、`nice -n 19` 起一条 cargo，输出落日志。
- `apply_row.py`：把 `crates/mutations.tsv` 第 N 行的替换打到自己的拷贝上（原文必须恰好命中一次）。
- `restore.sh`：把拷贝里的一个文件还原成冻结副本的。
- `l4.sh`：M3-c，在 `recovery.rs` 上打 `mut/` 里的读者改法（r1、r2、r3a、r3b）或第 3668 行，换新旧两版 `core_review_…` 测试文件，跑水位用例。`core_review_old.rs` 是 `git show HEAD:` 取出的旧版。
- `l2.sh`：M2，把 `walk.rs` 旋钮支换成「只判最右之前 k 片」，跑 `checker_known_bad_images` 的过滤集。
- `probe_m1_ring.rs`、`walk_probe_patch.rs`、`probe2.sh`、`probe3.sh`：M1 版本级探针；`probe3.sh` 是单线程的那版（`probe2.sh` 用 4 线程，共用进程环境变量，读数串了，作废）。`probe.sh` 是第一版（debug，太慢，放弃）。
- `logs/`：全部原样输出。`logs/probe.txt` 是 M1 引用的探针读数。
