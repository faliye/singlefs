# 辩方腿（sonnet）模型目录：m3-batch10-code-r3

全部在冻结副本的拷贝 `/tmp/claude-1000/m3-batch10-code-r3-sonnet/work/`（取自 `/tmp/claude-1000/m3-batch10-code-r3/tree/`）上做，只跑 harness 档，命令都经
`run-with-memory-cap.sh 8G` 与 `capped.sh 15`。副本上的数不进 kb。

- `probe-rollback-direction.diff`：在 `rollback_by_a_forward_publish.rs` 的「接着覆盖写」循环里每写一次就查一遍：只许 I-3.1 红、且已分配减遍历大于 0。`probe-direction-per-state.txt` 是 60 个状态（三条慢档各 20 个 txg，32 到 51）的读数；`run-rollback-4-slow.txt`、`run1-full.log` 是四条慢档的结果。
- `arm-x-w-mount.diff`：`mount.rs` 重读分支的两个替代臂。臂 W = 整份换成重读那一遍（不设环境变量）；臂 X = 整份换成重读那一遍，另外第一遍自证过的根在重读那一遍不是自证过就拒（`ARM_X_REFUSES=1`）。`run-arm-w-*`、`runW-full.log`、`run-arm-x-*`、`runX-full.log` 是在 `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root` 上的结果。`runM-full-merge-baseline.log` 是入库那一版（按槽并）在三个测试二进制上的基线。
- `check-mutation-anchors.py`：核 `crates/mutations.tsv` 里第 10h 批 14 行与四行整行替换的原文在冻结副本里恰好出现一次。
- `runM2877-…`、`runM2873-…`、`runM2864-…`：三行变异（torn 慢档、丢盘整读、归盘改回落点公式）在副本上单行复跑的结果；变异用 `/tmp/claude-1000/m3-batch10-code-r3-sonnet/mut.py <行号>` 按 tsv 原文换进副本。
