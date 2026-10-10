# m3-batch33s-code-r1 正推腿模型与复跑脚本

- `rerun.sh`：不编译的几项（报告 0.1、0.2、0.7、0.8）。用法写在文件头第一行。`compare.sh` 比两次输出。
- `expected.out`：交回前在空目录跑出的原样输出。
- 要编译的几项（我在草稿目录里分条跑的，每条经 `run-with-memory-cap.sh`，编译 6G、跑 4G、`harness checker_known_bad_images` 8G，线程 7）：
  1. harness 基线：副本里 `cargo test --release --offline -p singlefs-harness --test checker_known_bad_images -- --include-ignored`（报告 0.3，203 条）。
  2. 变异行逐条：`SINGLEFS_REPO=<副本> SCRATCH=<草稿目录> PRISTINE_TREE=<冻结副本> SINGLEFS_MAIN_REPO=<主仓> bash mut_row3.sh <行号…>`（先把 `apply_row.py` 放到 `$SCRATCH/bin/`；每行套用、跑该行 cargo 参数、还原；被杀时副本里源码留着变异，下一趟开头先还原）。报告 0.9 的行号：3102–3110、257、950、2883–2885、2914–2917、2951–2956、3032、3038、3039、3064、3065、3069，另 2239、2240、2919（红）与 2983（编不过）。
  3. 内核表达式数与判法闭包：`sizes/` 是一个独立小工程（`Cargo.toml` 里的 `@CHECKER_TIER@` 换成副本里 `crates/singlefs-checker-tier` 的路径，`Cargo.lock` 取副本的），`cargo build --release --offline` 后跑 `kernel-sizes`（报告 0.5）与 `closure <副本根>`（报告 0.6）。
  4. GPU 变异行的编译与 naga 校验：`mut_wgsl.sh <行号…>`（用上一条的小工程）。
- 守卫仿真 `guard_model.py`、变异表对比 `cmp_mut.py`（合前表 `mutations-pre-s.tsv.gz` 是 git 暂存区 `:crates/mutations.tsv`，3121 行）。
