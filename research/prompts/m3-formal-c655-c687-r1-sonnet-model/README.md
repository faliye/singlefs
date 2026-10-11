# m3-formal-c655-c687-r1 云端正推（Sonnet）的探针与原样日志

这些是拷在冻结副本（`/tmp/claude-1000/m3-formal-c655-c687-r1/tree/`）上、不碰主仓的 harness 档小池探针。不入 workspace，不进门禁。

复现（在冻结副本的一份拷贝里）：

1. `y1_probe_appended_part.rs`：把 `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` 整份拷成 `crates/singlefs-harness/tests/y1_probe.rs`，把文件里每个 `#[test]` 换成 `#[allow(dead_code)]`，末尾接上这个文件。
2. `y3_probe_appended_part.rs`：同样拷 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 成 `y3_probe.rs`，换 `#[test]`，末尾接这个文件；`TMPDIR` 指到可写目录（`build_pool` 在里面建稀疏镜像文件）。
3. `allocator-fallback-counter.diff`：只给 `crates/singlefs-core/src/allocator.rs` 加两个静态计数器，数 `FallBackToLowestFreeSlot` 那一支走过几次（y1_probe 里 `y3_probe_fallback_*` 读它）。`patch -p0` 或手工加。
4. 编译与运行：`nice -n 19 bash scripts/capped.sh 10 cargo test --offline -p singlefs-harness --test y1_probe --no-run`；跑测试二进制经 `bash scripts/run-with-memory-cap.sh <上限> bash scripts/capped.sh 10 <二进制> <测试名> --nocapture --test-threads=1`；本会话里别的任务也在用内存包装时要设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由>`。

日志（原样，未改）：

| 文件 | 内容 |
|---|---|
| `y1-probe-run2.log` | Y1：六个场景只挂载，R1 对 R2 的根记录、树表条目、系统配置槽 |
| `y1-probe-run3.log` | Y1：系统配置世代号、tail 两条关系式，九个场景 18 个盘行 |
| `y1-probe-run4.log` | Y1：有树而文件集合为空的一版 |
| `y3-bad-run1.log`、`y3-bad-run2.log` | Y3：a、b、b2、c、d、d2、e1、e2、e3 各一份坏镜像上的红与 I-5.5 判定 |
| `y3-scan-run2.log` | Y3 f：单元区 240 / 256 / 384 槽 × 只暖机挂载 1–30 次再发第一个文件，池级 checker 判整份镜像（36 份）|
| `y3-fb-run1.log`、`y3-fb-run2.log` | Y3 f：同族历史上 `FallBackToLowestFreeSlot` 走过几次（准入开 42 份、准入关 18 份）|
| `walk-y3-patch.diff` | 对冻结副本 `crates/singlefs-checker/src/walk.rs` 的补丁：I-9.16 加 ③ ④，I-5.5 的 a、c、e3 改 `partially_judged`、d 改不报；打在拷出的副本上，不入库 |
| `y3-after-run1.log` | 补丁只含 ③ ④ 时 `y3_probe_bad_images` 的原样输出 |
| `y3-after2-run1.log` | 补丁全含时同一批坏镜像的原样输出 |
| `y3-after-known-bad.log`、`y3-after2-known-bad.log` | 两种补丁状态下 `checker_known_bad_images` 全部 69 个 |
| `y3-after-treetable-ordering.log` | `tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker`（补丁 ③ ④ 时）8 个 |
| `y3-after-pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.log`、`y3-after2-first-tree-building.log` | I-5.5 合法镜像那份测试，补丁 ③ ④ 时与全含时各一次，9 个 |
| `y3-after-checker_narrow_invariants_and_abandoned_roots.log` | 补丁 ③ ④ 时 20 个 |

说明：`y3-fb-run1.log` 用的 mounts 列表是 `{1,2,3,4,6,8,10,12,16,20,24,30,40,60}`，`y3-fb-run2.log` 用 `{1,4,8,12,24,60}`（追加段里留的是后一份）。

内存包装的几次非结果：第一次不带并行理由起的那一次退出码 252（同会话已有一条在跑，没跑）；之后几次 252 / 251（内存排不上；上限超过峰值表 2 倍被拒）都是包装自己的结局，没有留日志，换了上限（`auto` 或小上限）重跑。
