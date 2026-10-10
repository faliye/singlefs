# E142（新池新建文件的干跑） 第 22 次第二段：主 agent 给的开工前提（2026-10-05）

照跑前登记 `research/prompts/e142-r22-prereg.md` 第五节 5.4「第二段」那一条与第十一节 S22-dump2，第二段开工之前主 agent 写明三件。这份文件不是准入输入；写在第二段任何产物之前。

1. **续做第二段**：第一段交回（报告 `/tmp/claude-1000/e142-r22-run/report.md` 第五节）之后，问题单第 1 行已够判、第 3 行 (d) 由主 agent 整张表跑完已够判，只剩第 2 行（P8G）开着，续做第二段。
2. **`crates/` 的 P8G 导出给，命令是**：在主工作区（仓根）跑
   `bash research/scripts/run-with-memory-cap.sh <上限> cargo run -q -p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump -- --device-bytes 8589934592`
   ——与 `replay.sh` 的 `driver_e142` 第一条命令相同，只多 `-- --device-bytes 8589934592`。
3. **那条命令用到的 `crates/` 改动是谁做的**：删除线第 18 批 X（implementation-writer，报告 `/tmp/claude-1000/m3-batch18/x/report.md`，补丁 7645e4ae…），给导出 bin `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` 加了可选参数 `--device-bytes <每盘字节数>`，不带参数时两块盘各 `TEST_IMAGE_DEFAULT_BYTES`（4 GiB）、输出与只收零个参数时逐字节相同（文件第 14–16 行文档）。2026-10-05 合入主工作区，合入验证 `/tmp/claude-1000/merge-and-verify/2026-10-05-413718/`：159 个 harness 测试二进制全绿，fmt 与 clippy 退 0。执行员开工先自己核一次「不带参数的导出与第 22 次主产物导出那一半逐字节相同」，不相同就停在 S22-dump2，Q142.97 记「还差」。

依据：主 agent 的调度判断（删除线记录 `records/2026-10-01-删除与并发事务.md` 2026-10-05 那几条）。执行员第二段自己的修订（只许收严或补臂，规矩同登记第十二节）交回时写在报告里，由主 agent 追加到这份文件末尾，不另起文件。

## 执行员第二段修订（主 agent 2026-10-05 从执行员报告 `/tmp/claude-1000/e142-r22-run2/report.md` 第四节原样追加）（2026-10-05；时点：步 ⑥ 单测全绿、小表与命令六七跑完之后，第二段产物一次没跑之前）

规矩同登记第十二节：只收严或补臂与判据；登记第六、七、九节每一条的去留与报告方式一条没动。依据：登记 5.4 ⑥(i)–(vi) 与本文件第 1–3 条（主 agent 的开工前提）。

### 一、冻结值（第二段装置）

| 项 | 值 |
|---|---|
| 装置 `research/e7-index-bench/src/bin/e142_new_pool_file_creation_dry_run.rs` | 10866 行，sha256 `c400139b8cfc7c0e4b96d483200055e9891252c9171bbbeb06771046bf63d899` |
| 变异表 `research/mutations/e142_new_pool_file_creation_dry_run.tsv` | 210 行，sha256 `ef63ea858443d708868f3ca55659f62727e4c867d96b5c234bf4dbf3e1fcf1c3`（第一段冻结的 206 行里只改了 M55 一行的锚点与替换文，末尾追加 M228–M231 四行） |
| 第一段冻结值（核过，S22-sha 不触发） | 装置 10282 行 `4720213e…5230`、变异表 206 行 `17132e80…0aa`，开工时现取与登记第十二节相同 |
| 单测 | `--list` 118 条，其中 1 条 `#[ignore]`；`cargo test --release -p e7-index-bench --bin e142-new-pool-file-creation-dry-run`：117 过 0 败 1 忽略（第一段 107 过；新加 10 条） |

### 二、装置改了什么（登记 5.4 ⑥）

1. (i) 每盘字节数：`PoolParameters.device_bytes`（主点三个构造函数都取 `DEVICE_BYTES` = 4 GiB），`device_slots()`、`unit_area_slots()` 现算；`build_allocation_tree`、记账三个函数（`free_slot_count`、`free_run_count`、`empty_cluster_segment_count`）收单元区槽数参数；读路径 `read_allocation_tree` 经 `BlockReader::device_bytes()` 取池自己的每盘字节数（新增 trait 方法；`Pool` 加字段 `device_bytes`，`mkfs` 按参数填）。`DEVICE_SLOTS` 常量改成 `#[cfg(test)]`（只剩主点单测钉 262 144 用它）。
2. (ii) `PoolParameters::settled_two_devices_of_device_bytes`、`p8g_parameters()`（两盘、每盘 8 589 934 592，别的字段同 `settled_two_devices`）。
3. (iii) `main()` 在 `name=verdict` 之后加 P8G 一段，全部行以 `name=p8g_` 起头：`p8g_config` 1、`p8g_role_labels` 1、`p8g_write_list` 1、`p8g_device_region_bytes` 29（带 `region=`、十六进制全文）、`p8g_write_list_row` 29、`p8g_recover_full` 1、`p8g_journal_header_record_bytes` 3、`p8g_accounting` 1（字段同 `name=accounting`）、`p8g_gap`（只 G22）1、`p8g_layout_field` 174、`p8g_layout_field_summary` 1、`p8g_positive_control_main_geometry` 1、`p8g_r18_clause_fields` 3、`p8g_r18_sensitivity` 2、`p8g_verdict` 1，另 `p8g_control_main_point` 1。V22e 点名的五类数（29 / 174 / 1 / 1 / 1）不变；其余各类是为了让甲类里「扫打印出来的行」的那几个字段在 P8G 上有行可扫，行体与主点同名行同一种写法。
4. (iv) 判决字段分类（25 个，主点 `name=verdict` 里的次序）。判据照登记：值由主点两盘那条流的写、字节或读回（调用链里读 `settled_two_devices()` 那条流的输出）算出的归甲；输入是一盘对照、G 系别的点、层 0、或与流无关的常量自检的归乙；有一处说不清归甲。

| # | 字段 | 类 | 装置行（本文件冻结版） | 为什么 |
|---|---|---|---|---|
| 1 | `width_mismatches` | 乙 | 7159 `width_rows()` 一段 | 30 行宽度表是常量对常量，不经任何一条流 |
| 2 | `write_list_ok` | 甲 | 8132 | 数这条流窗口里的写 / 屏障 / FUA |
| 3 | `recover_full_ok` | 甲 | 7273 | 读回这条流写完的池 |
| 4 | `layer0_states_ok` | 乙 | 字面 `not_run` | 层 0 主臂枚举不跑 |
| 5 | `layer0_violations` | 乙 | 字面 `not_run` | 同上 |
| 6 | `control_states_ok` | 乙 | 8133 | 一盘无屏障对照流 |
| 7 | `control_violations_ok` | 乙 | 8134 | 同上 |
| 8 | `control_missed_zero` | 乙 | 8135 | 同上 |
| 9 | `control_false_alarm_zero` | 乙 | 8136 | 同上 |
| 10 | `control_outcome_matrix_ok` | 乙 | 8137 | 同上 |
| 11 | `g6_states_ok` | 乙 | 8142 | 一盘加屏障的 G6 点 |
| 12 | `g6_violations_zero` | 乙 | 8143 | 同上 |
| 13 | `g7_states_ok` | 乙 | 8144 | 一盘 FUA 当边界的 G7 点 |
| 14 | `g7_violations_zero` | 乙 | 8145 | 同上 |
| 15 | `positive_control_main_geometry_ok` | 甲 | 8146 | PC2 在这条流的写上逐单元 / 全持久恢复 |
| 16 | `role_label_contradictions_zero` | 甲 | 8147 | 扫这条流打印的行（含它的 `role_labels`、`write_list_row`） |
| 17 | `role_labels_all_defined` | 甲 | 8148 | 同上 |
| 18 | `text_number_mismatches_zero` | 甲 | 8149 | 扫这条流打印的行（含 G22 文本与记账数） |
| 19 | `text_numbers_all_classified` | 甲 | 8150 | 同上 |
| 20 | `named_instances_ok` | 甲 | 8151 | journal 记录角色名与 G22 里的空闲字节数，都取自这条流 |
| 21 | `journal_differing_states` | 乙 | 字面 `not_run` | 层 0 主臂枚举不跑 |
| 22 | `r18_system_configuration_fields_ok` | 甲 | 8042 | 读这条流写完的两份系统配置槽 |
| 23 | `r18_root_flags_ok` | 甲 | 8053 | 读这条流的根槽 |
| 24 | `r18_g8_ok` | 甲 | 8095 | 取这条流的 fsid、实例代号、写入代数（只在模型里，仍归甲） |
| 25 | `r18_g9_ok` | 甲 | 8122 | 取这条流发布出的根记录（`output.root`） |

甲类 12 个，乙类 13 个（#1、#4–#14、#21）；没有为少改而归乙：#1 说不清是否随盘大小变时先看了 `width_rows`（30 行常量表，没有一项读设备字节数），才归乙。`p8g_verdict` 打甲类 12 个，字段名与主点同名字段逐字相同，次序同主点 `name=verdict` 里的相对次序。

5. (v) 单测 10 条（见报告）：7.2 A5 的 P8G 槽数；记账行两个几何的空闲 / 全空段 / runs / 标签 6；P8G 的 t10 记账单元字节里第 3 / 4 / 10 / 13 / 14 行的 value（A4 偏移）；两个几何窗口写变的恰是 A6 那九个区域；174 行字段表两个几何都全等且 12 个甲类字段都为真；P8G 一段的行数与前缀；前缀改名；打印的 `name=verdict` 与甲类字段的比较函数分得出被翻转 / 缺失的字段；读路径按池的每盘字节数定根的格数；池与崩溃态镜像报出每盘字节数。主点原有单测的断言一条没改；8 处 `Pool { devices: … }` 字面量（测试里）为编过加了 `device_bytes: DEVICE_BYTES` 一个字段。
6. (vi) 变异：M228–M230 照登记第九节原样；M55 的锚点与替换文因函数多收一个参数而改写（`free_run_count(&occupied)` → `free_run_count(&occupied, parameters.unit_area_slots())`，变异内容不变）。

### 三、补的臂与判据（只收严）

1. **补一条 M231**：`root_cells_per_device_of_reader` 里读路径取每盘槽数写成主点常量（读路径按池的每盘字节数这一点，P8G 产物字节上看不出，P8G 的根里只有第 0 格有孩子）；红的单测 `read_path_root_cells_per_device_follow_the_device_bytes_of_the_pool`。表因此是 210 条（登记写的 209）；Q142.103 的「条数」按 210。
2. **补一格对照 `name=p8g_control_main_point`**：同一个 `stream_section` 在主点参数上再跑一遍，甲类 12 个字段与 `main()` 自己打印的 `name=verdict` 行里同名字段逐个比，`equal=12 unequal=0` 才算「P8G 一段的判决算法在主点上复现了主点的判决」；不等就是这一段的算法与主点不是同一个算法，Q142.99 不判（记还差）。对照读的是打印出来的文本，不读 Rust 变量（同产物自检的口径）。
3. **P8G 的 12 个甲类字段进 Q142.99 的方式**：`p8g_verdict` 每个字段对同一份产物 `name=verdict` 里的同名字段，用一条命令逐个比（命令写进报告）；不同的字段数 = 0 ⇒ 不翻面。

### 四、没动的

登记 7.2 A1–A6 与 B1/B1b 的值、`name=width` 30 行、174 行字段表的期望字面、主点所有 `name=` 行的写法（Q142.101 守着）、独立 bin 与它的变异表、`crates/` 任何文件。

### 五、依据说明

- 「导出 bin 的 P8G 命令」与「crates 改动是谁做的」两件按 `e142-r22-s2-revision.md` 第 2、3 条（主 agent 开工前提）执行；开工前自己核过：不带参数的导出（`/tmp/claude-1000/e142-r22-run2/dump-nodev.txt`，sha256 `79640e8e3d75db783364cbca85a988cc51e7e30dbed3b217e2138aafad9b6110`）与第一段 `dump-n22.txt` 逐字节相同，也等于第一段主产物最后 39 行。
- 快照 A2（`research/results/e142-r22-s2-crates-sha256-a2-2026-10-05.txt`，303 行）与第一段快照 C 不同：这两次之间别的线合了补丁（含 `crates/singlefs-core/src/allocator.rs`、`mount.rs`、`recovery.rs`、`instance_table_sweep.rs`，`crates/singlefs-harness/src/scenario.rs` 等）。不带参数导出的字节没变，是上一条的读数。
