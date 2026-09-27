# 实审 A4d 报告：分配记录树那一项取 K1 整棵树；C545 那一格改成准入先拒

时刻：2026-09-27 00:28 – 01:5x UTC（JST 09:28 – 10:5x）。规格 `/tmp/claude-1000/impl-rev-a4d/spec.md`。交补丁：副本 `/tmp/claude-1000/impl-rev-a4d/copy`（00:28:02Z 从主工作区取，那一刻我那 7 份文件的 sha256 在 `sha256-at-copy.txt`，交回前核过主工作区里仍逐字节相同）里改、证红、跑检查；补丁目录 `/tmp/claude-1000/impl-rev-a4d/patch/`。主工作区一个字没写。

## 一、结论

1. **第 1 件（K1）做了。** `admission.rs` 的 ckpt_cost 分配记录树那一项换成 K1：1 + Σ_盘 Σ_{L = 0 … 根层级 − 1}（⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1），S_L 取 `allocation_record_tree::span_in_slots_at_level`（它读 `singlefs_format` 的 812、169，没写字面量）。量表 8 格重跑：每格 diff ≤ 0，多扣的块数与 A4c 报告第二节量表 K1 那一列**逐格相同**（第二节表）。A4c 情形 ②③ 那两段历史按探针的抽签逐位重放进量表那份测试：K1 下不少扣；改回 K0 两段都少扣（2600 槽种子 13 第 22 步 12 / 8，4 GiB 种子 2 第 132 步 14 / 12，与 A4c 那两行同数）。
2. **第 2 件（C545 准入先拒）做了。** `prepare_the_version_publish` 在 settle 之前、对「有普通分配、准入判着」的发布先判「这次的单元落得下」：① 每块盘段外成对空槽 ≥ 数据单元数；② 每块盘没挡的槽 − 数据单元取走的 ≥ 普通分配里的 extent / inode 节点槽数 + ckpt_cost（K1）。两个数由 `allocator.rs` 随三种挡位（已分配、隔离、扣住）增量维护，读时 O(这次挂载开过的段数)。不够交回 `SpaceAdmissionRefused`。A4c 那两格（384 槽 80 次空发布写第 7 个单元；256 槽 40 次空发布写第 2 个单元）现在都是 `SpaceAdmissionRefused`、两块盘逐字节不变、分配器逐项不变，钉在规格给名的新文件里。
3. 每处改法都先红后改、留了变异：新追加 15 行、整行替换 2 行、删 8 行（锚点随 K0 函数一起没了的那 7 行与一行改了靶子的），证红 19 条全抓到（第四节）。
4. **会连带变红的测试不在我的清单里**（推的，没跑，第六节）：`second_transaction_step_one_overwrite.rs` 1 条、`second_transaction_supplement_two_unequal_devices.rs` 2 条，原来钉 `PlacementRefused`，准入先拒之后变成 `SpaceAdmissionRefused`。另有 4 个设计问题交主 agent（第七节）。

**什么现象会推翻它们**：
- 第 1 条：有一次发布写出的分配记录树节点不在「这一版之后那棵树」按位置能有的节点里（例如节点写到单元区之外的位置），或 `span_in_slots_at_level` 与 `AllocationRecordTreeGeometry::parent_of` 的分段对不上——K1 就不是上界。量表那份测试任何一格报少扣也推翻它。
- 第 2 条：有一段历史里增量维护的两个数与逐槽扫的对不上（`unblocked_slot_and_slot_pair_counts_are_maintained_incrementally_and_match_a_scan` 只罩了我走的那几种操作），或者有改三种位图的路径不经 `set_blocking_bit`（今天 grep 三张位图的写只剩这一个函数，第三节）。

## 二、第 1 件：分配记录树那一项取 K1

### 改在哪（行号是副本里改后的文件）

- `crates/singlefs-core/src/admission.rs` 第 732 行 `fn allocation_record_tree_positions_over_the_unit_areas(root_level: u8, allocator: &PoolAllocator) -> u64`：逐盘取 `unit_area_start()` 与 `unit_area_slots()` 定 [s, e)，逐层加位置数，加根 1。删掉 K0 的 `allocation_record_tree_nodes_on_two_leaf_paths_per_device` 与常量 `ALLOCATION_RECORD_TREE_LEAF_PATHS_PER_DEVICE_AT_MOST`。
- 第 657 行 `checkpoint_cost_of_the_version_to_build_on_with_node_capacities`：带文件的一版按根节点码 2 头现读的高 − 1 作根层级（D28 已定项 4「从根节点头里现读」）；树表 0 条、写过行的那一版（`None` 臂）取 `AllocationRecordTreeGeometry::of_allocator(allocator).root_level()`；没写过行的照旧 0。中央映射树那一项的删插数仍是「分配记录树那一项 + 记账树节点数」，跟着变。
- 模块文档、函数文档照改；原来文档里「⚠️ 射程：每块盘两条路径罩不住开新段 / 回落 / 换下很久以前的节点」那一段删了，换成 K0 为什么少扣、K1 为什么拿得准。

### 量表 8 格（`admission_checkpoint_cost_per_device_paths.rs`，debug，副本 01:0x UTC）

每格一行原样（`difference = 实写 − ckpt_cost`，多扣 = −difference）：

```text
name=a4b-checkpoint-cost-cell cell=two-1GiB empty_publishes=60 crossings=2 taller_central_mapping=0 central_mapping_grew=0 difference_min=-38 difference_max=-36
name=a4b-checkpoint-cost-cell cell=two-4GiB empty_publishes=60 crossings=2 taller_central_mapping=0 central_mapping_grew=0 difference_min=-529 difference_max=-527
name=a4b-checkpoint-cost-cell cell=two-64GiB empty_publishes=60 crossings=2 taller_central_mapping=0 central_mapping_grew=0 difference_min=-10339 difference_max=-10337
name=a4b-checkpoint-cost-cell cell=two-1TiB empty_publishes=30 crossings=2 taller_central_mapping=0 central_mapping_grew=0 difference_min=-167300 difference_max=-167298
name=a4b-checkpoint-cost-cell cell=two-4GiB-file-300-units-mapping-two-levels empty_publishes=60 crossings=5 taller_central_mapping=60 central_mapping_grew=0 difference_min=-530 difference_max=-528
name=a4b-checkpoint-cost-cell cell=two-4GiB-mapping-4-8 empty_publishes=60 crossings=2 taller_central_mapping=60 central_mapping_grew=0 difference_min=-882 difference_max=-877
name=a4b-checkpoint-cost-cell cell=two-4GiB-mapping-3-3 empty_publishes=60 crossings=2 taller_central_mapping=60 central_mapping_grew=0 difference_min=-1067 difference_max=-1057
name=a4b-checkpoint-cost-cell cell=two-4GiB-mapping-2-2-accounting-8-3 empty_publishes=60 crossings=1 taller_central_mapping=60 central_mapping_grew=1 difference_min=-145503 difference_max=-144961
```

| 格 | 多扣（量到） | A4c 报告 K1 那一列的多扣 | 同不同 |
|---|---|---|---|
| two-1GiB | 36–38 | 36–38 | 同 |
| two-4GiB | 527–529 | 527–529 | 同 |
| two-64GiB | 10337–10339 | 10337–10339 | 同 |
| two-1TiB | 167298–167300 | 167298–167300 | 同 |
| 4 GiB、文件 300 单元 | 528–530 | 528–530 | 同 |
| 映射压到 4-8 | 877–882 | 877–882 | 同 |
| 映射压到 3-3 | 1057–1067 | 1057–1067 | 同 |
| 映射 2-2 记账 8-3 | 144961–145503 | 144961–145503 | 同 |

这 8 格的区间钉进了用例（`PoolGeometryCell::over_reserved_slots_of_the_empty_publishes`，两条量表用例末尾的断言），不同就红。
改回 K0 时（证红日志 `prove-red-logs-harness/002.log`、`010.log`）量到的是 0–2、2–4、2–4、4–6、5–7 与 542–545、584–588、142819–142826：前五格与 A4c 报告 K0 那一列同数。

### 情形 ②③ 那两段历史（新用例 `empty_publishes_of_the_histories_where_one_device_rewrites_more_than_two_leaves_fit_in_the_whole_tree_checkpoint_cost`，第 1057 行）

把 A4c 探针 `probe_a4c_leaves.rs` 的抽签（xorshift64、同一个次序、同一组比重）搬进量表那份测试，重放两段：单元区 2600 槽、逼满、种子 13、200 步；两块 4 GiB、种子 2、135 步。断言：每段至少一次空发布一块盘改的叶多于两片；每次空发布每块盘实写 ≤ 之前那一版的 ckpt_cost。
K1 下原样（`measure-table-2.log`，debug 68 s）：

```text
name=a4d-history history=unit-area-2600-filled-seed-13 empty_publishes=144 more_than_two_leaves_on_one_device=1 refusals=0
name=a4d-history history=two-4GiB-seed-2 empty_publishes=82 more_than_two_leaves_on_one_device=2 refusals=0
```

改回 K0（`prove-red-logs-harness/001.log`）少扣的那几行原样：

```text
name=a4d-history-empty-publish publish=unit-area-2600-filled-seed-13#22/mount checkpoint_cost=8 fixed_point_slots_per_device=12 most_leaves_on_one_device=4
name=a4d-history-empty-publish publish=unit-area-2600-filled-seed-13#32 checkpoint_cost=8 fixed_point_slots_per_device=10 most_leaves_on_one_device=3
name=a4d-history-empty-publish publish=unit-area-2600-filled-seed-13#110 checkpoint_cost=8 fixed_point_slots_per_device=10 most_leaves_on_one_device=3
name=a4d-history-empty-publish publish=unit-area-2600-filled-seed-13#154/raise checkpoint_cost=8 fixed_point_slots_per_device=12 most_leaves_on_one_device=4
name=a4d-history-empty-publish publish=two-4GiB-seed-2#132 checkpoint_cost=12 fixed_point_slots_per_device=14 most_leaves_on_one_device=4
```

与 A4c 报告第二节那张表：2600 槽种子 13 第 22 步暖机 12 / 8、第 154 步抬 F 12 / 8，4 GiB 种子 2 第 132 步 14 / 12，同数。K1 下 2600 槽那一段与 K0 那一段分岔（式子多扣 4 块，准入拒与会话推抬 F 的时机跟着变；第 22 步那次挂载的暖机在 K1 下一块盘只改一片叶），一块盘改三片叶的那次落在第 195 步（探针副本里 release 扫过：`probe-k1-2600.log` 那一行 `first_over_two=Some("probe#195")`）；4 GiB 那段准入一次都没拒（`refusals=0`），第 81 步起就有改多于两片叶的空发布，K0 下少扣的是第 132 步。

### A4b、A4c 钉的用例照 K1 重钉（算式）

- `admission.rs` 单测 `allocation_record_tree_term_is_two_leaf_paths_on_each_device_plus_the_shared_root`（K0 的 [5, 9, 13, 13]）换成 `allocation_record_tree_term_counts_every_position_the_unit_areas_cover_below_the_root_plus_the_root`（第 1273 行）：单元区从槽 50176 起，叶 61 罩 [49532, 50344)。
  两块单元区 16 槽：只罩叶 61、根层级 1 ⇒ 1 + 2 × 1 = 3；两块 384 槽：叶 61、62 ⇒ 1 + 2 × 2 = 5；两块 2600 槽（[50176, 52776)）：叶 61–64 ⇒ 1 + 2 × 4 = 9；两块 1 GiB（[50176, 65536)，每块盘 ⌈65536 ÷ 812⌉ = 81 格、两盘 162 ≤ 169，根层级 1）：叶 61–80 ⇒ 1 + 2 × 20 = 41；两块 4 GiB（[50176, 262144)，根层级 2）：叶 ⌊262143 ÷ 812⌋ − ⌊50176 ÷ 812⌋ + 1 = 322 − 61 + 1 = 262、层级 1 ⌊262143 ÷ 137228⌋ − 0 + 1 = 2 ⇒ 1 + 2 × 264 = 529；三块 4 GiB ⇒ 1 + 3 × 264 = 793。
- 量表那份 `three_devices_rewrite_at_most_two_leaf_paths_on_each_device_and_the_shared_root`（第 1250 行）：`None` 臂的 ckpt_cost 从 3 × 2 × 2 + 1 + 树表 1 = 14 改钉成 793 + 树表 1 = 794（两格实改的节点仍是 7 与 10，都少于它）。
- `admission.rs` 单测里的常量 `CHANGES_OF_AN_EMPTY_PUBLISH_ON_TWO_FOUR_GIBIBYTE_DEVICES`（文档写「分配记录树至多 9 个节点」，K1 下不再成立）改名 `CHANGES_OF_TEN_REWRITTEN_MAPPED_NODES`、文档照实写，值不变（删 10 插 10，只是那两条中央映射树纯函数单测的输入）。
- 窄盘（240 / 256 / 384 槽）上 K1 = K0 = 5，A4b / A4c 在窄盘上钉的数（第 18、32、47、62 次、崩溃枚举的状态数 77 850 等）都不因第 1 件变。崩溃枚举那份只因第 2 件改钉（第三节末）。
- 窄盘上 K1 与 K0 同值、4 GiB 以上 K1 大几百到十几万块：凡是在 1 GiB 以上的盘上钉了 ckpt_cost、c_max 或式子可用值的用例都会变。我按 `grep -rln 'checkpoint_cost_of_the_version_to_build_on\|admission_reading\|SpaceAdmissionRefused\|ckpt_cost\|c_max' crates` 的 18 份文件逐份看过，钉了这类数的只在窄盘上（`second_transaction_supplement_two_admission_formula.rs` 用 240 / 256 槽），推的、没跑。

### 给书记员的 D28 已定项 4 原句（K1）

替换已定项 4「形态」一条里分配记录树那一半（今天的原文还是「Σ（每棵记录树当前的高）」，A4 / A4b 两次定案之后没写回）：

> 分配记录树按几何上整棵树计（用户 2026-09-27 定 K1）：1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1），S_L = 812 × 169^L；根层级是根节点码 2 头里现读的层级（树表 0 条、写过行的那一版取池几何的根层级）。它不读这一版有哪些节点：一次发布改几片叶的不动点收不了口（开放段装不下开新段、没有全空段回落、换下很久以前落在别处的节点，实审 A4c 量到一块盘改 3–4 片叶），拿得准的上界只有整棵树。代价随单元区线性涨：两块 4 GiB 盘 529 个节点，每块盘多扣约 2.7%（1 TiB 约 3.2%）。中央映射树照旧逐层按可能改的路径数与可能切出的节点数计，删插的条目数取「分配记录树那一项 + 记账树节点数」。

依据给书记员引：本报告第二节量表与两段历史的原样行；A4c 报告第二节。

## 三、第 2 件：C545 准入先拒

### 改在哪

- `crates/singlefs-core/src/allocator.rs`：
  - `DeviceFreeMap` 多三个字段（第 281 行一带）：`unblocked_slots`（三种位都没置的槽数）、`unblocked_slot_pairs_per_segment`（每个 64 槽段里两槽都没挡的偶数起点槽对数）、`unblocked_slot_pairs`（总数）。起步按段算（末尾不满一段、奇数槽不成对）。
  - 第 288 行 `enum SlotBlockingBit { Allocated, IsolatedByTheShadowLedger, HeldUntilFloorTakesEffect }`，第 425 行 `fn set_blocking_bit(&mut self, index, bit, is_set)`：改一槽一种位时按改前改后增量维护上面三个数。三张位图的写全改走它：`mark_allocated`、`mark_reclaimed`（回收）、`isolate`（隔离）、`clear_isolation_of_slot`、`hold_until_floor_takes_effect`（扣住）、`release_holds`（放开扣住，第 615 行：原来整张 `fill(false)`，改成只走 `held_per_segment` 非 0 的段、逐槽清）。核法：`grep -n 'allocated\[index\] =\|isolated\[index\] =\|held_until_floor_takes_effect\[index\] =\|\.fill(false)' crates/singlefs-core/src/allocator.rs` 只剩 `set_blocking_bit` 里那三行。
  - 第 456 行 `unblocked_slots()`、第 468 行 `unblocked_slot_pairs_outside_the_cluster_segments(&BTreeSet<SlotNumber>)`：总数减聚簇段那几段各自的数，O(开过的段数)，不扫单元区。
- `crates/singlefs-core/src/admission.rs`：第 465 行 `PlacementRoomOnOneDevice`（每块盘两个数，`of_every_device_of_the_allocator`）、第 496 行 `UnitsOfAPublishToLand`、第 511 行 `units_of_a_publish_to_land(普通分配那一半的角色, ckpt_cost)`、第 549 行 `admit_the_units_landing_on_every_device`（① ② 逐盘合取）。
- `crates/singlefs-core/src/transaction.rs`：
  - 第 3999 行 `struct OrdinaryAllocationPartOfThePublish` 与第 4060 行 `PublishPlan::ordinary_allocation_part`：把 `resolve` 开头那一段（inode 树写入、extent 树计划、切一单元事务）抽出来，`resolve` 改调它（行为不变）；它的 `rewritten_roles()` 给普通分配那一半的角色（数据单元、extent 节点、inode 叶容器、inode 根）。
  - 第 4890–4915 行：settle（第 4916 行）之前取普通分配那一半，`JudgedByTheFormula` 且有普通分配时调 `admit_the_units_landing_on_every_device`，不够 `map_err(PublishError::SpaceAdmissionRefused)?`。ckpt_cost 用 `pool.code_two_tree_node_capacities()`（写入口装的那一档）。关掉准入的开关下不判。
  - 式子那一判（第 4950 行，settle 之后）没动。`PublishError::SpaceAdmissionRefused` 与 `prepare_the_version_publish` 的文档补了这一判。
- `mount.rs`、`mounted_session.rs` 没动：挂载那几次发布（写行、暖机、抬 F、预演）都没有普通分配，这一判不走；会话照旧对 `SpaceAdmissionRefused` 推抬 F 再判。

### 钉成的几格（新文件 `admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs`）

| 用例（行） | 盘面 | 结局 |
|---|---|---|
| `a_write_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_by_the_space_admission_before_any_write_while_the_byte_formula_admits`（159） | 384 槽，崩了再挂，80 次空发布，顺序写到 6 单元；段外 0 对、段内 79 对，式子可用 ≥ 需求上界 | 写第 7 单元 ⇒ `SpaceAdmissionRefused`，每块盘 available 0、demand 14 槽；盘上逐字节不变、分配器逐项不变 |
| `a_write_on_the_narrower_pool_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_for_its_data_units_before_any_write`（241） | 256 槽，崩了再挂，40 次空发布；段外 0 对、段内 48 对，式子可用 −14 槽 | 写到 2 单元 ⇒ `SpaceAdmissionRefused`，报的是落得下那一判（available 0、demand 4 槽），不是式子的 −14；盘不变、分配器不变 |
| `a_write_whose_data_unit_lands_but_whose_commit_generated_units_find_no_unblocked_slot_is_refused_by_the_space_admission_before_any_write`（295） | 384 槽，第一个文件之后除段外最低一对之外没挡的槽全标已分配 | 覆盖写 ⇒ `SpaceAdmissionRefused`，available 0、demand = 4 + ckpt_cost 槽；盘不变、分配器不变 |
| `with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write`（394） | 第一格的盘面，会话的准入用只供测试的开关关掉 | 写第 7 单元：推它的是 `PlacementRefused { Data(0), NoFreeSlotOnAnyDevice }`，推一串抬 F 之后做成，F 抬上去，checker 0 违例（「准入放行而落点取不到也推」那一条今天只剩这条路测得到） |

`second_transaction_admission_raises_the_floor_before_refusing.rs` 里 A4c 那条 `…_is_refused_by_placement_before_any_write_while_the_byte_formula_admits` 与它的两个辅助函数搬走了（上表第一行是它改成准入先拒之后的样子）。同一份文件里：
- `an_admitted_overwrite_whose_data_unit_finds_no_slot_raises_the_floor_and_is_published_in_the_session` 改成 `an_overwrite_whose_data_unit_has_no_slot_pair_outside_the_cluster_segments_is_refused_by_the_admission_and_published_after_raising_the_floor_in_the_session`（第 119 行）：70 次覆盖写，推过抬 F 的那几次整表钉住——(18, 段外 38 对)、(32, 10)、(47, 2)、(62, 0)，推它们的都是 `SpaceAdmissionRefused`；第 62 次段外 0 对，是落得下那一判拒的（此前第 62 次是 `PlacementRefused`）。
- `after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes`：长大被拒那一条断言从 `last_refusal` 是 `PlacementRefused` 改成 `SpaceAdmissionRefused`；(3, 4) 那两个数没变。
- 模块文档一句照实改。

崩溃枚举那份（`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`，只改钉值）：第 62 次直接调发布路径那一条断言从 `PlacementRefused` 改成 `SpaceAdmissionRefused`（第 82 行），注释与文档照实改；推的那一串不因交回的成员变。探针副本里把用例截在状态数断言之后、release 跑（`probe-crash-count.log`）原样：

```text
CRASH-COUNT states=77850 writes=53 root_writes=3 publishes=3
CRASH-COUNT overwrite-after-raise-ok
```

与 A4c 报告第五节同数。全量枚举没跑（标了 ignore，归提交时的崩溃验证员）；没加新的崩溃枚举用例，不用登记 `crash-case:`。

## 四、变异行与证红

补丁目录里三份（格式照 `research/scripts/apply-writer-patch.py` 文件头）：`mutations-append.tsv` 15 行、`mutations-replacements.tsv` 2 行、`mutations-delete.txt` 8 个名。主工作区上 `apply-writer-patch.py <补丁目录> --dry-run` 核过（第五节）。

证红一律 `bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-a4d/copy --memory 8G <crate> <名…>`（经 `capped.sh 5`；副本的 `crates/mutations.tsv` 先按这三份删、换、追加）。基线（不改源码、同组参数）都绿：core `test result: ok. 1 passed; 0 failed; …; 129 filtered out`，harness `test result: ok. 1 passed; 0 failed; …; 9 filtered out`（`prove-red-logs-*/baseline.log`），基线红集为空。行的参数带测试名前缀过滤，所以「同时红了哪些」只在过滤剩下的那几条里看：每条都只红了点名的那一条。

| # | 变异（改坏哪一行 → 改成什么） | 红的断言（原样的消息头） | 日志 |
|---|---|---|---|
| 追加 1 | `admission.rs` 带文件那一臂 `allocation_record_tree_positions_over_the_unit_areas(…)` → 盘数 × 2 × 根层级 + 1（K0） | 量表 `empty_publishes_of_the_histories_…` 第 1112 行 `空发布每块盘写出的固定点槽数多于之前那一版现算的 ckpt_cost（保留池少扣）`（五行见第二节） | harness/001 |
| 追加 2 | 同上 | `every_empty_publish_under_the_node_format_capacities_…` `每格多扣的块数与钉住的（实审 A4c 量表 K1 那一列）对不上`（量到 0–2 / 2–4 / 2–4 / 4–6 / 5–7） | harness/002 |
| 追加 3 | `None` 臂同样退回 K0 | `three_devices_…` `left: MetadataBlocks(14)` `right: MetadataBlocks(794)` | harness/003 |
| 追加 4 | K1 `for device_map in &allocator.devices` → `.iter().take(1)` | core 单测 `allocation_record_tree_term_counts_every_position_…` | core/001 |
| 追加 5 | K1 `for level in 0..root_level` → `0..1` | 同上 | core/002 |
| 追加 6 | `THE_ROOT_COVERING_THE_WHOLE_KEY_SPACE: u64 = 1` → `0` | 同上 | core/003 |
| 追加 7 | `transaction.rs` `if units_landing_is_judged {` → `if false {` | 新文件第一格 `顺序写到 7 个单元：准入先拒（这次的数据单元在段外落不下）：Err(PlacementRefused { unit: Data(DataUnitIndexInFile(0)), refusal: NoFreeSlotOnAnyDevice })` | harness/004 |
| 追加 8 | `admission.rs` `if room_on_the_device.slot_pairs_for_data_units < units.data_units {` → `if false {` | 新文件 256 槽那一格 `顺序写到 2 个单元：准入先拒…：Err(PlacementRefused { unit: Data(…0), refusal: NoFreeSlotOnAnyDevice })` | harness/005 |
| 追加 9 | 同上 | 会话那条 `推过抬 F 的那几次覆盖写：都是准入拒的；…`（第 62 次那一项变成 `PlacementRefused`） | harness/006 |
| 追加 10 | `(slots_left_after_the_data_units < units.commit_generated_slots).then(` → `false.then(` | 新文件第三格 `覆盖写：准入先拒（提交内生块落不下）：Err(PlacementRefused { unit: ExtentRoot, refusal: NoFreeSlotOnAnyDevice })` | harness/007 |
| 追加 11 | 同上 | core 单测 `landing_needs_slot_pairs_…`（第三格放行） | core/004 |
| 追加 12 | `.checked_sub(slots_of_the_data_units)` → `.checked_sub(0)` | 同上（第三格 19 ≥ 15 放行） | core/005 |
| 追加 13 | `.checked_add(checkpoint_cost.0)` → `.checked_add(0)` | core 单测 `units_to_land_are_…`（12 变 4） | core/006 |
| 追加 14 | `release_holds` 里 `self.set_blocking_bit(index, SlotBlockingBit::HeldUntilFloorTakesEffect, false);` → 只清位图 | core 单测 `unblocked_slot_and_slot_pair_counts_…`（「放开扣住」那一步增量数与扫的对不上） | core/007 |
| 追加 15 | `mounted_session.rs` `SpaceAdmissionRefused(_) \| PlacementRefused { .. } => true` → 准入拒 false、落点拒 true | 会话截断那条 `第 9 次覆盖写：Publish { cause: SpaceAdmissionRefused(…) }`（在造盘面那一步就红，比写回早） | harness/008 |
| 替换 1（原第 760 行「实五 P3pl …落点被拒原样交回」） | 原样的变异，靶子换成新文件关掉准入那一格（原靶子那条用例在准入先拒之后走不到落点被拒） | `经会话顺序写到 7 个数据单元，推过抬 F 之后做成：Publish { cause: PlacementRefused { unit: Data(…0), refusal: NoFreeSlotOnAnyDevice }, floor_raises: [] }` | harness/009 |
| 替换 2（原第 1125 行「实审 A4c C545 那一格…」） | 原样的变异（用户数据候选不排除聚簇段），靶子换成 allocator 单测 `user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed`（原靶子搬走了；准入先拒之后那一格在落得下那一判就拒，这条变异在它上面活着） | allocator 单测红 | core/008 |

另把两条既有行在副本里重证了一遍（名字与原文都没动，只是量表那两条用例加了钉值）：「实审 A4b：中央映射树那一项退回每层一个节点…」「实审 A4b：压小容量下量 ckpt_cost 照旧按节点格式的容量算」，都抓到（harness/010、011，`每格多扣的块数与钉住的…对不上`）。
没重证的既有行（第 1033 行以外点名量表的 A4b 行、第 761–777 行点名会话那份的其余几行）：我没动它们的锚点与靶子测试的判法，留给提交时的 59 号。

删的 8 个名：原第 761 行（「实五 P3pl（D3 已定项 9 第 2 条的界）…落点被拒原样交回」：准入先拒之后截断那条走不到落点被拒，与替换 1 同一个变异、同一个新靶子就重复了）；原第 974–978、1031、1032 行（K0 的函数与常量没了，锚点零命中；它们防的「按盘分路 / 两条路径 / 根」由追加 1–6 接住）。

clippy 之后改了三处名字与一个 `match`（第五节），动到了追加 7–10、14、15 与替换 1 的靶子测试的代码（判法没变），这 7 行在改完之后又重证了一遍，结果在第五节末。

## 六、会连带变的测试（不在我的清单里，推的，没跑）

我的清单外的测试二进制按规矩没跑。下面是按代码读出来的推断，**推的**，主 agent 派人跑或改：

1. `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs` `publish_running_out_of_space_midway_leaves_the_allocator_as_it_was`：盘上只留 50182–50183 给数据单元、别处全占，钉 `PlacementRefused { ExtentRoot, NoFreeSlotOnAnyDevice }`。准入先拒之后 ② 先拒（数据单元取走那一对之后没挡的槽 0 < extent 根 + inode 叶容器 + inode 根 + ckpt_cost），交回 `SpaceAdmissionRefused` ⇒ 这条红。`crates/mutations.tsv` 第 12 行「失败的发布不退回分配器」点名它：② 在动分配器之前拒，那条变异在这条用例上可能活下来。
2. `crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs`：
   - `filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking`：小盘写满，钉覆盖写 `PlacementRefused { Data(0), SomeDevicesFullDeviceSetSelectionUndefined }`。① 在小盘上段外 0 对 ⇒ `SpaceAdmissionRefused`（只报盘 1）⇒ 红。第 76、77、145、149 行点名它。
   - `devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written`：小盘只剩 196638–196640 三槽，钉 `PlacementRefused { ExtentRoot, CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined }`。① 小盘段外 1 对够，② 小盘 3 − 2 = 1 < 4 + ckpt_cost ⇒ `SpaceAdmissionRefused` ⇒ 红。第 78、80 行点名它。
   - `user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written`：空间充裕，①② 都过，仍是 `PlacementRefused { …UserDataSlotsDiffer… }`，不变。
   改法两条路，交主 agent 定：这几条原意是测落点那一道（小盘满、各盘去处不同），在它们的分配器上装 `SpaceAdmission::SkippedByTheTestOnlySwitch`（与我给会话那条对照用例同一个做法）就照旧走到落点；或照准入先拒改钉成 `SpaceAdmissionRefused`。D3 已定项 8 那一句「小盘没有全空段而大盘有时，在动任何状态之前拒绝（发布层统一报假性 ENOSPC）」与后一种读法对得上。
3. 随机历史那份（`second_transaction_supplement_three_random_history.rs`）是门禁 74 号的二进制，登记给我，跑了（第五节）：22 过、2 红，红的两条与 A4c 报告第七节记的基线红同名。准入判着、256 槽那一段断言「落点那一道走不到」（`PlacementRefused` 计 0 次），准入先拒之后只会更少。
4. 别的二进制里钉 `PlacementRefused` 的（`grep -rn PlacementRefused crates/singlefs-harness --include=*.rs` 列出的那几份）：`second_transaction_supplement_two_commit_generated_fallback.rs`、`a_floor_raise_refused_for_space_counts_as_short_of_space.rs` 拒的是抬 F 那一串空发布（没有普通分配，这一判不走），`second_transaction_supplement_two_root_ring_turn_in_one_mount.rs` 装了关掉准入的开关——推的，不变。
5. 1 GiB 以上的盘上钉了 ckpt_cost、c_max 或式子可用值的用例：按第二节那条 grep 逐份看过，没找到（钉值的都在窄盘上）；4 GiB 上保留池与切换预留每块盘被扣的从 164 槽涨到 6 989 槽（A4c 报告第二节量表那一列的算法：13 × c_max + 8），写得很满的 4 GiB 用例可能更早被式子拒——没逐条核，推的。
6. `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`（实验装置，按 `SpaceAdmissionRefused` 分支计数）：准入先拒会让原来记成落点被拒的那几次记成准入拒，它的变异表与产物要不要重跑，归 E156 那一路，我没碰。

## 七、交主 agent 的设计问题（停在那一处，没自己定）

1. **② 的读法**：规格写「这次固定点单元（按 K1 的最坏数）落不落得下」。我取的是：提交内生块要的槽 = 这次普通分配里的 extent / inode 节点（照计划精确数，它们也按提交内生块的规则落）+ ckpt_cost（K1）。两处条款没写：
   - 要不要把普通分配里的 extent / inode 节点算进去（我算了；不算的话它们先把槽吃掉、固定点照样可能落不下）；
   - 有普通分配的发布，固定点比一次空发布多（数据单元、extent / inode 节点的映射 key 也要删插，中央映射树可能多改几片），ckpt_cost 是空发布的上界，不是这一次的上界。要补的条款原句（建议）：「有普通分配的一次发布，固定点上界 = 分配记录树那一项 + 记账树节点数 + 中央映射树那一项（删插数取分配记录树那一项 + 记账树节点数 + 这次普通分配进映射的单元数）+ 1」。条款定之前，这一格走到时照旧在落点那一道、任何写之前拒（`PlacementRefused`），盘上不变。
2. **② 只数槽**：两槽的 inode 叶容器要 32768 对齐成对、各盘答的落点要同槽（开段各盘一致、回落各盘同槽），今天的落点规则里有，这一判不判——条款没写落得下那一判要不要判它们。今天的样子：放行之后落点照样可能拒（`PlacementRefused` 的 `CommitGeneratedPlacementsDiffer…` 等），在任何写之前。
3. **`SpaceAdmissionRefused` 的载荷一型两用**：落得下那一判拒时，`DeviceShortOfDemand.available` / `demand` 装的是「短的那一种单元落得下的字节 / 要的字节」，不是式子的可用(d) / 需求(d)（函数文档写了两种怎么取）。改类型要动 `second_transaction_supplement_two_admission_formula.rs`（它按字面构造 `AdmissionRefusedOnSomeDevices` 与 `DeviceShortOfDemand`，不在我的清单里），所以没改。要不要给两种判据各一个成员（或给载荷加一个「哪一判」的字段），交主 agent 定、另派。
4. **会连带红的三条用例**（第六节第 1、2 条）怎么改，与它们点名的第 12、76、77、78、80、145、149 行变异要不要换靶子。

### 两件只列不做

- **A4c 第三节第 4 条：抬 F 回收之后「扣住」的槽式子没扣。** 现状：`admission.rs` `AdmissionReading::of_allocator` 文档那条 ⚠️ 还在（读数里「已分配」按占着的槽数，扣住的槽在记账上已回空闲、分配器却不发，式子里没有一项装它们）。这一轮的落得下那一判把扣住当挡住（`unblocked_slots` 不算它们），所以「放行之后因为扣住而落不下」那一格现在在准入先拒那一道就拒；式子那一判仍把它们当可用。要问的：扣住的槽要不要进 D28 已定项 1 的式子（单列一项，还是并进「已分配」），还是只靠落得下那一判兜着就够——前者改条款与记账口径，后者要在条款里写明「式子放行不等于落得下」。
- **A4c 第三节第 5 条：`df` 把段内空槽算空闲会不会是 D3 已定项 9 第 1 条的假性 ENOSPC。** 现状：`crates/` 里没有 `df` 的实现（`grep -rnE 'fn [a-z_]*(df|statfs|reported_free)[a-z_]*' crates --include=*.rs` 零命中），`df` 报什么只在 D16 已定项 1 的条款里。C545 那一格段外 0 对、段内 79 对：段内那 79 对在这次挂载里对用户数据关着（D3 已定项 8 第 2 条），准入先拒只改交回的成员，不改这一点。要问的：`df` 的空闲要不要扣掉「这次挂载开过的聚簇段里的空槽」（扣了 `df` 少报、随挂载内开段数变；不扣则这一格按第 1 条字面是假性 ENOSPC），以及会话在这一格推抬 F 推到上限仍落不下（聚簇段只在重开之后才对用户数据开放）要不要多一种收尾（例如推一次卸载 / 重开，或放开段内空槽给用户数据）。

## 五、第 4 步那几样（副本里，01:2x–01:3x UTC；开跑前 `ps` 看到别的会话的 cargo（`/tmp/claude-1000/m2-closeout-code-r1-opus/` 与 `/tmp/claude-1000/e158-r4-device/` 两处编译、测试），没有 qemu / fio / vm-bench / e152 测量；每条经 `capped.sh 5`，跑编出来的代码经 `run-with-memory-cap.sh 8G`）

动到的测试二进制（整个二进制，debug）与 core 单测，末行原样（`progress.md`）：

```text
final core-lib exit=0 3s: test result: ok. 130 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.21s
final admission_checkpoint_cost_per_device_paths exit=0 65s: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 65.28s
final second_transaction_admission_raises_the_floor_before_refusing exit=0 98s: test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 98.58s
final second_transaction_crash_inside_the_floor_raise_pushed_by_the_session exit=0 0s: test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
final admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments exit=0 1s: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.26s
```

基线（改之前，同一份副本 00:40 UTC）：core 127 过；量表那份 3 过；会话那份 11 过；崩溃枚举那份 1 ignored。

`cargo fmt --all -- --check`：退 1，差异全在别人的文件：

```text
$ grep '^Diff in' final-fmt.log | sed -E 's/:[0-9]+:$//' | sort | uniq -c
     67 Diff in /tmp/claude-1000/impl-rev-a4d/copy/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
```

`cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `check.sh` 那 7 条：第一次红在我写的三处（allocator 单测闭包参数 `pool` 遮蔽、新文件闭包参数 `device_map` 遮蔽、会话那份一个 `match` 的 `other =>` 通配臂），改了（改名、`match` 换成两个 `matches!`）。第二次整仓 `clippy exit=101`，红全在别人的文件：`e156_allocation_basis_counts.rs` 6 处、`e158_root_choice_repair.rs` 10 处（都是 `shadow_unrelated`）。只对我动到的目标：`clippy core exit=0`（`-p singlefs-core --all-targets`）、`clippy mine exit=0`（我那 4 个测试目标）。

`cargo build --offline --all-targets`：`Finished \`dev\` profile … in 21.98s`、`build exit=0`。

登记给我的门禁阶段（副本里跑，副本没有 `.git`；74 号带 `SINGLEFS_GATE_FULL=1`、`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`），退出码与判定行原样：

```text
33-mutation-tables exit=0   ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1128 条的原文各命中源码一次；…
53-format-const-placeholders exit=0   ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）
92-layout-checker-sync exit=77   ! /tmp/claude-1000/impl-rev-a4d/copy 不是 git 仓，本阶段跳过
94-checker-implementation-disjoint exit=0   ✓ checker 与实现只共享常量模块 `singlefs-format`（…checker 的 4 份源码零处引 `singlefs_core`…）
93-feature-bits exit=0   ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，…）
89-closeout-row27-preconditions exit=77   ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）
74-model-differential exit=1   ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
```

- 33 号判的是副本里合并之后的表（1128 条）。主工作区的表在这之间被别的会话追加了 11 行 E158（`sha256-at-copy.txt` 里只有 `crates/mutations.tsv` 对不上），`apply-writer-patch.py --dry-run` 在主工作区上核过：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1144 行`。
- 92 号在副本上退 77（不是 git 仓），89 号退 77，都按没判写。
- 74 号红，**不是这一轮带来的**：在还原成取副本那一刻的副本（`copy-probe`）上同一个二进制 release 跑一遍当基线（`baseline-random-history.log`），也是 `test result: FAILED. 22 passed; 2 failed; 2 ignored`，红的同样是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`（快档「历史 96 段：跑完 67、以已知红收尾 {0: 1}、新发现 28」，新发现是 I-7.4）。改后同一个二进制（`after-random-history.log`）逐段汇总行与基线逐行相同（`diff` 只差末行用时 66.15s / 47.63s）。A4c 报告第七节记的也是这两条。我没修。

补丁：`git apply --check` 在主工作区（01:34 UTC）上干净；我那 6 份已有文件在主工作区里与取副本时逐字节相同（`sha256sum -c sha256-at-copy.txt` 6 行 OK）。`git apply --check --stat` 原样：

```text
 crates/singlefs-core/src/admission.rs              |  405 +++++++++++++--
 crates/singlefs-core/src/allocator.rs              |  258 +++++++++-
 crates/singlefs-core/src/transaction.rs            |  125 ++++-
 .../admission_checkpoint_cost_per_device_paths.rs  |  539 +++++++++++++++++++-
 ...n_admission_raises_the_floor_before_refusing.rs |  276 +++-------
 ...inside_the_floor_raise_pushed_by_the_session.rs |   14 -
 ...t_land_outside_the_mounts_clustered_segments.rs |  444 ++++++++++++++++
 7 files changed, 1754 insertions(+), 307 deletions(-)
```

主工作区的 `git diff --stat -- crates litmus` 此刻是别的会话的改动（`mutations.tsv`、`e156`、`e158`、`checker_narrow_invariants_and_abandoned_roots.rs`、`second_transaction_step_five_reuse.rs`，末行 `5 files changed, 6534 insertions(+), 175 deletions(-)`），不含我的：我一个字没写主工作区。

clippy 改名之后动到的 7 行变异重证（`prove-red-logs-core-2/`、`prove-red-logs-harness-2/`）：

```text
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了        （core：放开扣住那一行）
✓ 点名 6 条：跑了 6 条，跳过 0 条，跑的都抓到了        （harness：追加 7、8、9、10、15 与替换 1）
```

基线都绿（`1 passed; 0 failed; … 129 filtered out` 与 `1 passed; 0 failed; … 9 filtered out`）。第一轮的全部 19 行：`✓ 点名 8 条：跑了 8 条，跳过 0 条，跑的都抓到了`（core）、`✓ 点名 11 条：跑了 11 条，跳过 0 条，跑的都抓到了`（harness，含重证的两条 A4b 旧行）。

## 八、这一轮写过的文件

补丁 `/tmp/claude-1000/impl-rev-a4d/patch/crates.patch`（7 份，`git apply --check --stat` 见第五节）：

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-core/src/admission.rs` | K1（第 732 行新函数，删 K0 的函数与常量）；ckpt_cost 两臂改调它；落得下那一判的类型与两个函数（第 465–594 行一带）；模块与函数文档；单测：K0 那条换成 K1 那条、改名一个常量、加两条 |
| `crates/singlefs-core/src/allocator.rs` | 三个增量计数字段、`SlotBlockingBit`、`set_blocking_bit`、两个读函数；六处位图写改走 `set_blocking_bit`；`release_holds` 改成只走有扣住的段；单测加一条 |
| `crates/singlefs-core/src/transaction.rs` | `OrdinaryAllocationPartOfThePublish` 与 `PublishPlan::ordinary_allocation_part`（`resolve` 改调它）；`prepare_the_version_publish` 在 settle 之前加落得下那一判；`PublishError::SpaceAdmissionRefused` 与函数文档 |
| `crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs` | 8 格多扣区间钉值与两条断言；两段历史的重放（抽签、起步、重放）与一条新用例；三块盘那一格改钉 794；模块文档 |
| `crates/singlefs-harness/tests/second_transaction_admission_raises_the_floor_before_refusing.rs` | 会话那条改名改钉（整表）、加两个辅助函数；A4c 那条 C545 用例与两个辅助函数搬走；截断那条一句断言；模块文档；`use` 跟着删 |
| `crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs` | 只改钉值：第 62 次那条断言 `PlacementRefused` → `SpaceAdmissionRefused`，注释与文档 |
| `crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` | 新建，4 条用例（第三节表） |

变异表（不在 `crates.patch` 里，按名字合并）：`patch/mutations-append.tsv` 追加 15 行（名字见第四节表「追加 1–15」，全名以文件为准）；`patch/mutations-replacements.tsv` 替换 2 行（原第 760 行「实五 P3pl（D16 已定项 1「准入」那一行：准入放行而落点取不到也推）：会话只在准入拒时推抬 F，落点被拒原样交回」、原第 1125 行「实审 A4c C545 那一格（D3 已定项 8 第 2 条 / 已定项 10 ②）：…」，名字不变、只换靶子）；`patch/mutations-delete.txt` 删 8 个名（第四节末）。

草稿目录里另有：`report.md`（本报告，`patch/report.md` 是它的一份拷贝）、`progress.md`、`spec.md`（主 agent 给的）、`sha256-at-copy.txt`、`copy-time.txt`、几份跑命令的小脚本（`run-*.sh`、`build-mutation-rows.py`）、日志（`baseline-*`、`final-*`、`gate-*`、`measure-*`、`probe-*`、`prove-red-*`、`after-*`）、`probes/`（探针源码与说明，只在副本里跑过，不入库）、`old-c545-test.txt`（搬走之前那条用例的原文）。

## 九、没做什么

- 没走三方对抗；没提交；主工作区一个字没写（交补丁）。层 0、QEMU、herd7 与 crates 变异整表（59 号）归崩溃验证员，都没跑；全量 `cargo test`、`check.sh`、`gate.sh` 整轮没跑。
- 崩溃枚举那份（标了 ignore）全量没跑，只按改后的前缀在探针里算了状态数（77 850，与 A4c 同）。
- 我的清单外、会连带红的三条用例（第六节）没跑、没改；它们点名的第 12、76、77、78、80、145、149 行变异没重证。
- 规格「两件只列不做」那两件只写了现状与问题（第七节），没改代码。
- ② 里 inode 叶容器成对、各盘同槽、有普通分配时固定点可能多于 ckpt_cost，这三格条款没写，没判（第七节第 1、2 条）。
- `transaction.rs` 式子那一判（第 4950 行）仍按节点格式的容量取读数（A4c 第四节那一处），规格没点它，没改；落得下那一判的 ckpt_cost 按写入口装的容量取。
- 门禁 92 号在副本上退 77（不是 git 仓），89 号退 77，都没判；打进主工作区之后要再跑。33 号在主工作区上要等补丁打进去再判一次（`apply-writer-patch.py` 打的时候会跑）。
- 随机历史只按 A4c 那两段重放了两个种子（外加探针里 2600 槽 8 个种子、4 GiB 1 个种子扫过），别的盘宽与种子没扫。

## 十、清理

交回前删掉的仓副本（大小是删之前 `du -sh` 的）：`/tmp/claude-1000/impl-rev-a4d/copy`（16G，带自己的 `target/`）、`/tmp/claude-1000/impl-rev-a4d/copy-probe`（1.4G，带自己的 `target/`）、`/tmp/claude-1000/impl-rev-a4d/base`（7.7M，取副本那一刻的 `crates/` 与 `litmus/`，生成补丁用）。别的编译目录没建。
