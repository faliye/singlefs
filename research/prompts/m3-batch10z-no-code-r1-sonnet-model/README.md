# m3-batch10z-no-code-r1 正推腿的模型与日志

都建在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree/`（被判文件 sha256 对 `research/prompts/m3-batch10z-no-code-r1-snapshot/crates-sha256.txt` 336 行全 OK）的拷贝上，不进仓内 crates。拷贝与编译目录交回前已删，清单与大小在报告末尾。

- `src/s2_probe.rs.txt`：放进拷贝 `crates/singlefs-harness/tests/s2_probe.rs`。起点是上一轮（lm-r1）正推腿的 `src/s2_probe.upstream-lm-r1.rs.txt`，加了：`both_of`（分配位与隔离位同时置着的槽数，即新访问器）、`c_unblocked_of`（根环表里还记着的 C 落点盖住的槽里三位都没置的个数）、`c_handed_out_of`（C 落点盖住的槽被这个进程新发出去的记录压着的个数）、动作 `U`（正常卸载）、故障 `rroot`（R 的根槽读回全零）、环境变量 `S2_NOCHECK`（不跑池级 checker，快 20 倍）、`S2_WATCH_SLOT`、`S2_DEBUG_CU`、`S2_DEBUG_HO`（调试输出）。
- `src/core-debug-edits.diff`：拷贝上对 `singlefs-core` 的调试改动（`DeviceFreeMap::debug_bits` 读三位；`S2_DEBUG_CU` 开着时在 mount.rs 里打 `CLOOP`、`ALOOP`、`SRC1544*` 行）。行为不变，不进库。
- `scripts/`：`piece.sh`、`piece-old.sh`（经内存包装起一件探针，新树 / 改前树）、`mutate.py` + `mutants_extra.py`（对 tree-mut 套 `allocator.rs`、`mount.rs` 的变异，名字里用 `+` 叠两个）、`run-mut.sh`（套变异、编探针、跑 7 批形）、`repo-mut.sh`（套变异、编入库的 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 用例、含 ignored 跑）、`rowrun.py`（套实现员第十四块变异行、按行里的命令跑点名的用例）、`scanmut.py` + `scan.sh`（扫源码模型）。
- `gpu/`：`repo-rows.json`（实现员第十五块的变异行）、`n14-rows.json`（第十四块的 12 行）、`my-rows.json`（我加的 9 个显存账变异）、`scan-extra.json`（扫源码的绕过写法 N1）、`gmut.py`（对 `gpu_memory_ledger.rs` 套变异、编库单测、跑 `gpu_memory_ledger`）。
- `scan-model/`：`src/main.rs.txt` 改名 `main.rs` 编；把第十五块 checker 档两条扫源码用例逐字搬出来（`the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking`、`every_layer_handed_the_stop_request_hands_it_on_or_asks_it`），喂任意一份源文件，报 GREEN / RED。
- `logs/`：`runs/<批>/<件>.log`、`runs/M/<变异>.sum`（变异 × 批形汇总）、`runs/RM/*.log`（入库用例文件在各变异上的输出）、`runs/EV/*`（报告引的改前改后各一件）、`gpu-lib-baseline.log`（123 条库单测）、`repo-tests-none.log`。

重建：`rsync -a <冻结副本>/ tree-core/`；拷 `src/s2_probe.rs.txt` 成 `tree-core/crates/singlefs-harness/tests/s2_probe.rs`；`patch -p0` 或手工按 `src/core-debug-edits.diff` 改 core；`CARGO_TARGET_DIR=… bash research/scripts/capped.sh 8 cargo test --release --offline -p singlefs-harness --test s2_probe --no-run`；`scripts/*.sh|py` 里的 `W=`、`$HOME/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet` 要改成自己的草稿目录。
