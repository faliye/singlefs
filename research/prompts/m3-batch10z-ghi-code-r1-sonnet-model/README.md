# m3-batch10z-ghi-code-r1 正推腿（Sonnet）模型与探针

全是这一条腿自己写的，不进库的产品代码；只在冻结副本的拷贝上编、跑 harness 档、core 与自己写的模型，没跑 checker 档测试、没起 GPU 设备。
冻结副本：`~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree/`（被判文件 sha256 与 `research/prompts/m3-batch10z-ghi-code-r1-snapshot/crates-sha256.txt` 逐项核过 OK：mount.rs、allocator.rs、gpu_memory_ledger.rs、gpu_unit_checks.rs、crash_amplification.rs）。

## 文件

- `probe_rollback_revived.tail.rs`：K2-1 探针（管理员回退当 R）。用法：在冻结副本的拷贝里
  `cp crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs crates/singlefs-harness/tests/probe_rollback_revived.rs`，
  把 `use singlefs_core::mount::{` 那一段里的 `rollback_floor_ceiling, unmount,` 改成 `rollback_floor_ceiling, roll_back_by_a_forward_publish, unmount,`，
  再把本文件整份追加到 `probe_rollback_revived.rs` 末尾；跑
  `bash scripts/capped.sh 15 bash scripts/run-with-memory-cap.sh 8G nice -n 19 cargo test --offline -p singlefs-harness --test probe_rollback_revived -- probe_units_revived --nocapture`。
- `probe_started_now_blocks.rs`：K4-2 探针，放进 `crates/singlefs-harness/tests/` 跑同样的命令（`--test probe_started_now_blocks`）。
- `probe_cost_of_the_protected_list_scan.rs`：K2-2 耗时探针，是 `crates/singlefs-core/src/allocator.rs` 测试模块里的一条用例（插在 `fn pool_after_mkfs` 之前），
  命令 `cargo test --offline -p singlefs-core --lib -- probe_cost_of_the_protected_list_scan --nocapture`。
- `scan-model/`：把 checker 档四条扫源码用例的逻辑原样搬出来，运行时从 `CA_SRC`、`GU_SRC` 读源文件，用来对变异后的源码复跑（不依赖 checker 档包）。
- `tools/mutate_rows2.py`：把背景材料改动全文里的变异表行（harness、core 两档）逐行套上、跑那一行点名的测试、还原；`tools/mutate_custom.py` 套自己写的变异（`*.json`）；
  `tools/scan_mutate.py` 对 scan-model 套 checker 档两份源码的变异（表里的行加自己的 `scan_extras.json`）。脚本里的路径是这条腿草稿目录的路径。
- `logs/`：上面各条跑出来的原样输出。
