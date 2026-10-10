# 模型目录：正推腿的探针（只读导入，不入实现）

- `apply_probes.py <仓副本根>`：往仓副本的 `crates/singlefs-checker-tier/tests/` 加探针（`zz_probe_support/mod.rs`，每条流打一行 `ZZPROBE`：写数、段数、段形、整张写表（带屏障）的摘要、基镜像摘要、展开 6/28/30/全域/甲二 的态数）。层 0 一侧在每个 crash_enumeration_* 测试文件末尾追加 `zz_probe_layer0_*`；放量一侧改 `common_crash_points/mod.rs` 与 through_unmount、through_rollback 两个测试文件（打 `AMP …` 行）。
- `probe-l0.log`：这一轮跑出的层 0 一侧原样输出（5 个测试二进制，筛选词 `zz_probe`，每个经 `run-with-memory-cap.sh 6G`）。放量一侧没跑（见报告「要主 agent 跑的命令」）。
- `cite-lines.py`：报告引文行号现查用的小脚本（读 `标签|文件|原文` 清单，打命中数与行号）。
- 起法（主 agent 在仓副本里跑）：
  cd <仓副本>；`CARGO_TARGET_DIR=<编译目录> ROCKSDB_LIB_DIR=<某个已编好的 librocksdb-sys 的 out 目录> ROCKSDB_STATIC=1`（省掉现编 RocksDB 的 C++；本机 `target/release/build/librocksdb-sys-f7ce5abb7da4f505/out` 里有 librocksdb.a）。
  编：`bash research/scripts/capped.sh 7 cargo test --offline --release -p singlefs-checker-tier --features verdict-store --test <目标>… --no-run`（仓副本里 1 分 19 秒编完，用 7 线程）。
