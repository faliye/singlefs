# 正推腿（Sonnet）m3-batch57-code-r1 的模型与日志说明

目录里的东西（均为我的草稿副本上的做法，冻结副本一个字没改）：
- `probe_k1_mixed_knob_tests.rs`：两条探针测试（K1 旋钮历史中途改变）。用法：把这个文件的内容追加到冻结副本拷贝里
  `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的末尾，`cargo test -p singlefs-harness --test checker_known_bad_images -- sonnet_probe_k1 --nocapture`。
- `mutate_walk.py` / `run_mutant.sh`：把冻结副本的一份文件改一处写进拷贝、跑点名的测试、还原。
- `run_row.py`：按 `crates/mutations.tsv` 的行号跑那一行（锚点命中次数、必须红的测试是否红）。
- `mutate_k3_mapping_first.py`、`mutate_k3_gen_before_count_only.py`、`mutate_k2_callsites_skip_product.py`：K3、K2 的变异。
- 日志在上一层 `~/.local/share/singlefs/work/m3-batch57-code-r1-sonnet/`：`base-*.log` 基线、`mut-*.log` 变异、`row-<行号>.log` 变异表行复跑。
