# C834 正推腿的模型目录

- `patch/mount-c834-variants.patch`：对冻结副本 `crates/singlefs-core/src/mount.rs` 的补丁，只在草稿副本里用：把 `placements_in_the_accounts_of_newer_readable_roots` 里「取过号的实例代号不比那条根新」那一句换成按环境变量 `C834_VARIANT` 选的候选读法（k1 今天的实现、k2 逐盘取最大再盘间取最小、k2_min_over_every_slot 各盘各槽取最小、k3 要有带更大号的根或记录、k4 照 I-7.7 第 ② 句、k5 跨盘跨槽取最大、k6 在 k3 上再加映射根槽里是更新实例的自证节点；k6_only_mapping_root、k6_records_ge、k6_mapping_ge 是 k6 的三个变异）。不进产品代码。
- `tests/c834_order_of_devices_decides_the_newer_root_judgement.rs`：放进冻结副本拷贝的 `crates/singlefs-harness/tests/` 跑的测试（三条：逐史打印结局、后一次挂载的记录落在 R 之上时 R 的单元不丢、结局表）。
- `tools/rerun.sh`：照 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps`；每个候选各跑一遍，摘要与 `out/summary.txt` 逐行比。
- `out/`：每个候选的完整日志（`all-*.log`）、摘要（`summary.txt`）、入库已有用例在 k6 下的结果行。
