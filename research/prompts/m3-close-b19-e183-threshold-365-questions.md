# 问题单：E183 第一段装置的清扫阈值占位常量跟上实现默认值 365（批 19，2026-10-30）

这份单不是为岔路建的，是 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 第一段装置的一处常量镜像跟不上实现，要重出登记产物。

## 背景（主 agent 的观测，2026-10-30）

- 批 19 照用户定案（C690 = 365）把实例表清扫阈值的默认值定为 365 行：`crates/singlefs-core/src/instance_table_sweep.rs:74` `pub const DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE: Self = Self(365);`。
- E183 第一段装置 `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:78` 仍是 `const SWEEP_THRESHOLD_PLACEHOLDER_ROWS: u64 = 369;`（文档注释第 77 行指 C690 那一行「阈值是占位值 369」，那一句已在 2026-10-30 写回里改成 365）。全文件只在第 1879 行把它拿去与实现默认值比（`constants_against_the_implementation` 的 `sweep_threshold_placeholder_rows` 一项），H4 的各项读数不读它（`grep -n SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 只命中 78、1879 两行）。
- 两条 checker 档快档用例因此红（另一个会话 2026-10-30 的快档与这一轮第三趟快档都报）：`local_constants_equal_the_implementation_constants`「sweep_threshold_placeholder_rows：本地 369 实现 365」；`segment_one_all_output_equals_the_registered_product_byte_for_byte`（登记产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` 第 6 行「E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=369 implementation=369 equal=true」，复跑登记 `scripts/experiment-apparatus/replay.sh` 第 1811 行 `exact`）。
- 第 4209 行 `SWEEP_THRESHOLD_CANDIDATE_ROWS: [u64; 4] = [64, 365, 369, 738]` 是第三段 H5 扫的四个 θ，与这一处无关，不动。

## 要回答的问题

| # | 问题 | 怎么算答完 |
|---|---|---|
| 1 | 常量改成 365 之后，第一段 `all` 产物除第 6 行那一行比对之外还变不变 | 重出产物，与 2026-10-23 那一份逐行 diff；只变第 6 行就答「只变那一行」，别的行变了逐行列出、并判 H4 的判决（实验页「判决」一节整行抄的那几行）变没变 |
| 2 | 改后两条快档用例与 A31 独立核对（`driver_e183_anchor_check`）是不是都对得上新产物 | 新产物登记进复跑表，产物 sha 的钉值照新产物改，两条用例与独立核对的命令写进交回报告「要主 agent 跑的命令」 |
