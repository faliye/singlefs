# E156（alloc-basis 四条岔路的代价数） 第 5 次重跑的问题单（2026-10-05）

<!-- doc-lint:not-numbers PC A10 W7 -->

用户 2026-10-05 定「core 在分配器没装根环表而记了一条根时断言」（弹窗选项原文「再加 core 断言」；起因是 E171 流三的 I-3.1 红，调查报告 `/tmp/claude-1000/e171-s1-i31-investigation/report.md`），第 31 批 31-1 实现它（报告 `/tmp/claude-1000/m3-formal-31-1/report.md` 第 9 节）。E156 装置 `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 有两处因此在第一次记根时 panic：W7 的阳性对照 PC-分配器（第 4732 行起，`HhAllocatorSource::WithoutRootRingPositiveControl`，经第 1420 行 `allocator_without_the_root_ring`：「换回不装根环表的分配器，环转过一圈之后 A10 应判红」），与内联单测 `new_pool_file_creation_state`（第 5052 行，7 处调用）。`scripts/replay.sh` 第 193 行 `E156|@driver_e156||e156-alloc-basis-counts-2026-09-26-r2.out|exact` 会因此对不上。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 「不装根环表的分配器」这一臂今天写不出来（core 断言），W7 那一格要证的「A10 分得出不回收」还要不要、换成什么对照才证得出同一件事 | 换一种不经分配器 API 造「不回收」的对照臂 / 改成断言 panic 的对照 / 撤掉这一臂并写明它证的东西由 core 断言接管 | 新对照在不回收的那一形上 A10 不红，或新对照与原对照判的不是同一件事 | 新对照写定、在装置上跑出 A10 红（或撤臂的依据写定） | 开着 |
| 2 | 内联单测与主路径照新 `build_pool` 装根环表之后，E156 第 4 次已够判的读数（岔路单各行与产物 `research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 的判决字段）还逐个相同吗 | 相同 / 哪几格不同 | 判决字段或计数有一个变 | 主产物重跑、逐字段比完 | 开着 |
