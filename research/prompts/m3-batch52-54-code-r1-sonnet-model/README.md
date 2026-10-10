# m3-batch52-54-code-r1 云端正推腿的模型目录（Sonnet）

全是对冻结副本 `~/.local/share/singlefs/work/m3-fixes-code-r1/repo/` 的拷贝（`~/.local/share/singlefs/work/m3-batch52-54-code-r1-sonnet/repo/`，只拷了 crates、litmus、.cargo、Cargo.toml、Cargo.lock）做的探针与变异；冻结副本一个字没改，拷贝的 24 份被判文件加 `crates/mutations.tsv` 的 sha256 与 `research/prompts/m3-batch52-54-code-r1-snapshot/crates-sha256.txt` 逐份相同（`sha256sum -c` 全 OK）。

- `tools/mut.py`、`tools/run_mut.sh`：inode_tree.rs 的九个变异（m1–m9）与三个目标测试。路径写死在草稿目录 `W=~/.local/share/singlefs/work/m3-batch52-54-code-r1-sonnet`，要复跑先把 `W` 改成自己的拷贝；`W/pristine/` 放变异前的原件。
- `tools/walkmut.py`：walk.rs 的十一个变异（w1–w11），测试集在 `tools/S.txt`。基底是带探针的 walk.rs（`patches/walk-probe-container-numbers.patch` 加在冻结副本上就是 `W/pristine/walk_with_probe.rs`）。
- `tools/rowrun.py <变异行名前缀>`：照 `crates/mutations.tsv` 一行的锚点、替换文与点名用例，在拷贝上改、跑、改回。
- `patches/walk-probe-container-numbers.patch`：walk.rs 末尾加的单测 `sonnet_probe_container_numbers::probe`，直接调私有函数 `judge_lineage_or_retirement_container_capacity_of_a_version` 看各种容器号 / 记录数组合的判定。
- `patches/inode-tree-probe-left-absorb.patch`：inode_tree.rs 测试模块加的 `sonnet_probe_last_leaf_pair_fits_only_on_the_left`（[117,117,117] 删最后一片的一条）。
- `patches/reference-test-event-counters.patch`：给第 53 批的独立参照用例加事件计数（右吸收、左吸收、连锁、借、借后吸收、删空摘片、两对都装得下），结束时打一行 `EVENTS ...`。
- `logs/`：各次运行的原样输出；`logs/walkmut-results.txt`、`logs/inode-tree-mutation-results.txt` 是各变异结果的汇总（抄自当时终端输出）。
- 没写进工具的一个变异（mt1，树 ID 水位也只取没被抛弃的根）：把 `crates/singlefs-core/src/mount.rs` 第 8131 行起的
  `let tree_identifier_watermark = ring_roots\n        .iter()` 换成 `let tree_identifier_watermark = ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table)\n        .iter()`，
  结果见 `logs/inode-tree-mutation-results.txt` 末段。

所有 cargo 命令形态：`CARGO_TARGET_DIR=<自己的> nice -n 19 bash research/scripts/run-with-memory-cap.sh 10G bash research/scripts/capped.sh 12 cargo test --offline --no-fail-fast -p <包> ...`。没跑 checker 档包 `singlefs-checker-tier` 的测试。
