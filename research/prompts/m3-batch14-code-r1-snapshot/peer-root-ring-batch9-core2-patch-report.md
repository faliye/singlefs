# 第 9 批 9-5 core 簇实现员报告（2026-10-04）

批次：`records/2026-09-30-里程碑三开工第一轮.md#9`（9-5 core 簇）。交补丁：补丁目录 `/tmp/claude-1000/m3-r1/batch9-core2/patch/`（`crates.patch`、`mutations-append.tsv` 22 行、`mutations-replacements.tsv` 5 行）。副本取派发那一刻的主工作区；`scripts/apply-writer-patch.py <补丁目录> --dry-run` 对今天的主工作区判过：「✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 2325 行」。补丁没动 `crates/singlefs-checker/src/`。

## 结论

| 条 | 做成什么 | 证红 |
|---|---|---|
| 第 1 条 RS1 | 进程内重读读得出而自证不过与仍读不出同一处置（改法 A，按根环表记的最大 txg 算）；重读读出有效根的那一槽算进「这个进程知道住着根的槽」。抬 F 那一遍走同一个分类函数，一起改了 | 3 行变异 prove-red 抓到 |
| 第 2 条 RS2 | 补了「读得出有效且在窗口里」「读得出而被抛弃」「写行之后判有效用写行那一张」「树表 0 条那一版上的暖机记根走到重读」四条会红的用例；`mount.rs` 写行零单元那一处与 `transaction.rs` 写行那一处没有会红的用例（第七节第 1、2 条写明为什么） | 4 行变异 prove-red 抓到 |
| 第 3 条 RS3 | 准入抬 F 的上限把窗口里挂载那一刻读不出、之后没写过的槽当作可能是一条非空有效根，F 不越过它（新函数 `rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount`，挂着时的两处调用点都换成它）；探针那一形 X 读不出时上限 21（改之前 22），X 读得出时 21 | 1 行变异 prove-red 抓到 |
| 第 4 条 R2 | harness 判法 `newer_publish_witnessed_on_the_crash_image` 改取 R2，另写一份读槽（不调实现）；某块盘一份自证过的系统配置都没有那一形照实现的那个成员按真（第七节第 4 条） | 2 行变异 prove-red 抓到 |
| 第 5 条 清扫器 | 新模块 `crates/singlefs-core/src/instance_table_sweep.rs`：判（根环每槽、候选行、认领规则、跳过环里可读根的账、读失败与设备不全）、抹（抹前重读、FUA 写零头所在的物理块、每块盘一道屏障）、清扫发布（事务号 0、整链重写、落点被拒这一轮不删）；挂在 `mount.rs` 的 `establish_instance` 末尾、交回之前；开关与阈值是配置（阈值占位 369，C690）；后台那一支给了入口（判、抹前重读、发三段都能单独调），调度没做 | 9 行 harness 变异 prove-red 抓到；checker 档 3 行留给提交时 |

一句话：五条都落了代码与会红的用例；RS2 里两处记根点、「写行之前用最新根那一张」两样做不出会红的用例（走不到或看不出），第七节逐条写明；清扫器有几格条款没写，我取了不改盘的那一边并在第七节列出，交主 agent。

什么现象会推翻：主 agent 提交时跑 checker 档快档，`crash_points_of_the_instance_table_sweep` 的快档用例或 `crash_injection` 新加的那条库单测红；或 `checker-tier-crates-mutation-replay` 复跑这 22 行有一行没红。

## 一、这一轮写过的文件

改的（都在「要动的 crates 文件」那一行里）：

- `crates/singlefs-core/src/allocator.rs`：`RereadOfARootRingSlotUnreadableAtMount` 换含义换名（`NoValidRoot` → `AbandonedByTheInstanceTable`，`StillUnreadable` → `StillUnreadableOrNotSelfVerified`）；`RootRingOccupancy::ring_slots_known_to_hold_a_root` 收 `ValidRootReadWhenTheSlotUnreadableAtMountWasReread`；几处文档注释照新条款改。
- `crates/singlefs-core/src/mount.rs`：`reread_of_a_root_ring_slot_unreadable_at_mount` 的分类（RS1）；`roots_of_the_ring_for_the_ceiling` 另交回这一遍读坏、当没有根的槽；新 `rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount` 与 `ring_slots_unreadable_at_mount_by`，`rollback_floor_ceiling` 改成调它、给空表（签名不动：`on_device_modes.rs`、`e158_root_choice_repair.rs` 与一份 harness 用例还在调它）；准入抬 F 两处调用点（`raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk`、`raise_the_floor_through` 的 `Admission` 那一支）换成新函数（RS3）；`MountOutput::instance_table_sweep`、`InstanceStart::instance_table_sweep`、新公开入口 `mount_writable_with_instance_table_sweep_configuration`，`mount_writable_with_every_test_only_switch` 多一个参数，`establish_instance` 末尾调清扫（第 5 条）。卸载那一格（`newest_valid_root_txg_on_every_device` 那一段）没动。
- `crates/singlefs-core/src/lib.rs`：`pub mod instance_table_sweep;`。
- `crates/singlefs-harness/src/mount_transparency.rs`：第 4 条（第五节逐处列）。
- `crates/singlefs-checker-tier/src/crash_injection.rs`：`WritableMountPhase` 加 `ErasingHeadsOfTheInstanceTableSweep`、`InstanceTableSweepPublish` 两段（`ALL` 6 个）；`WritableMountOnTheCrashImage::publishes_before_the_instance_table_sweep` 与 `SecondCrashInputs` 同名一项；新公开函数 `publishes_before_the_instance_table_sweep_of_the_mount_output`、`writes_of_the_instance_table_sweep_phases`；`writable_mount_phase_of_each_write` 多一个参数、划出两段；库单测加一条。

新建的（都在清单里）：

- `crates/singlefs-core/src/instance_table_sweep.rs`（连 3 条库单测）
- `crates/singlefs-harness/tests/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs`（7 条）
- `crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs`（2 条）
- `crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs`（2 条）
- `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs`（9 条）
- `crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs`（3 条：1 条快档、2 条 `#[ignore]` 全量）

清单里没用到的：`crates/singlefs-core/src/transaction.rs`、`instance_table.rs`、`make_filesystem.rs` 一个字节都没改。

`crates/mutations.tsv`（不进 `crates.patch`）：

- 追加 22 行（`mutations-append.tsv`），名字逐行：`第9批 RS1：进程内重读读得出而自证不过改回当没有根…`、`第9批 RS1 代价那一形：…`、`第9批 RS1：重读读出有效根的那一槽不算这个进程知道住着根的槽…`、`第9批 RS2 探针 p1：…`、`第9批 RS2 探针 p2：…`、`第9批 RS2：写行那次发布的表定下来之后不换…`、`第9批 RS2 探针 q1：…`、`第9批 RS3：…`、`第9批 RR8：harness 判法复算判据 N-配置 回到取甲…`、`第9批 RR8：…不认某块盘一份自证过的系统配置都没有…`、`第9批 清扫 C597 判别力「不抹就删」…`、`第9批 清扫 C597 判别力 F5「不抹就删」…`、`第9批 清扫认领规则…`、`第9批 清扫：不跳根环里可读根的账…`、`第9批 清扫：读一个落点读失败当它不是单元…`、`第9批 清扫：不判全部设备在线…`、`第9批 清扫后台那一支：抹之前不重读…`、`第9批 清扫发布被落点拒不归…`、`第9批 清扫：开关关着时不看阈值…`、`第9批 清扫：现行那一版树表 0 条时不报第一版不支持…`、`第9批 清扫两段：抹头之后不按 FUA 分清扫发布…`、`第9批 清扫两段：WritableMountPhase::ALL 里没有抹头与清扫发布…`（全名见那份文件第一列）。
- 整行替换 5 行（`mutations-replacements.tsv`，名字照旧、只改原文或替换文跟上新代码）：`C378（认了）：取号之后写行或暖机报块设备错时…`（原文与替换文里 `InstanceStart` 字面多一项 `instance_table_sweep,`）、`三·2 Q-J：可写挂载的段表里没有续做…`（`ALL` 6 个 → 去掉 `Continuation` 剩 5 个）、`拒绝合法（11-4）：每块盘最新那一槽系统配置都报读错时 N-配置 复算成假…`（锚点换到 R2 写法里「判不出 ⇒ 按真」那一处）、`E2 进程内记根时挂载时读不出的槽成了窗口里最小的那个也不重读…`、`E2 单测：挂载时读不出的槽成了窗口里最小的那个也不重读`（替换文里的成员名跟着改名）。这 5 行在今天的主工作区里与我的开工副本逐字相同（合并前现核过）。

`git diff --stat -- crates litmus`（副本里，相对开工那一刻的主工作区）原样：

```
 crates/mutations.tsv                               |  32 +-
 .../singlefs-checker-tier/src/crash_injection.rs   | 241 +++++-
 .../crash_points_of_the_instance_table_sweep.rs    | 655 +++++++++++++++
 crates/singlefs-core/src/allocator.rs              |  51 +-
 crates/singlefs-core/src/instance_table_sweep.rs   | 902 +++++++++++++++++++++
 crates/singlefs-core/src/lib.rs                    |   1 +
 crates/singlefs-core/src/mount.rs                  | 167 +++-
 crates/singlefs-harness/src/mount_transparency.rs  | 131 ++-
 ...e_table_sweep_at_the_end_of_a_writable_mount.rs | 822 +++++++++++++++++++
 ...udges_the_system_configuration_witness_by_r2.rs | 163 ++++
 ...s_not_pass_a_slot_unreadable_since_the_mount.rs | 227 ++++++
 ...s_not_self_verify_counts_as_still_unreadable.rs | 866 ++++++++++++++++++++
 12 files changed, 4169 insertions(+), 89 deletions(-)
```

主工作区里我改的五个源文件此刻与开工副本的 sha256 前 16 位逐个相同（`mount.rs` 717e108a2229d567、`allocator.rs` e25aa237288d338d、`lib.rs` c1d87f3fa3609ca6、`mount_transparency.rs` b322a7755ecd0486、`crash_injection.rs` 46bfe50442a6dd99）；`crates/mutations.tsv` 主工作区多了 12 行别人的，按名字合并不撞。

## 二、第 1、2、3 条：用例与形状

RS1 / RS2 的用例都在 `root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs`（文件名是派发给的；RS2 那几条也放在这一份里，名字只说了 RS1 那一格——要改名的话交主 agent）。用例自带一个读包装 `DeviceWithOneRootRingSlotFaulted`：一个根环槽的读按四档坏（照读 / 每次报设备错 / 第一个写发出之前报设备错 / 只把下一次读的一个字节取反），common 模块不在清单里，没往里加。

| 用例 | 盘面 | 钉住什么 |
|---|---|---|
| `a_valid_root_read_bad_once_on_the_reread_is_counted_at_the_highest_txg_its_slot_can_hold` | 攻方 `e2_reread_flip 4 20 2 flip`（S = 4，实例 1 写到 20，槽 (0, 2) 住 txg 18，实例 2 挂载那一刻读错；发 txg 29 那一次只把一次读坏） | 那一次正是重读；重读之后那一槽照旧按 18 记着；txg 29 那一版 I-3.1 等五条不红 |
| `a_valid_root_read_on_the_reread_inside_the_window_holds_the_reclaim_threshold_at_its_txg` | 同上，发 29 那一次重读读得出 | 那一槽换成重读读出的有效根 18、算知道住着根的槽；txg 29 那一版不红 |
| `a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write` | 同上，之后那一槽每次读报设备错，准入抬 F 抬到上限 | `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot }`，录制流不多一步、盘面逐字节不变 |
| `a_front_hole_unreadable_at_mount_counts_more_allocated_only_on_the_version_that_rereads_it` | 攻方 `front_hole_eio 4 9 mount` | 判成什么钉什么：覆盖写到 9 + 2 × 3S，只有 txg 20 那一版红、红的只有 I-3.1（记账比遍历多，安全方向，与攻方 `out-fixA/fh-s4-h9-mount.out` 同一版）；txg 20 那一刻那一槽照旧按 9 记着 |
| `an_abandoned_root_read_on_the_reread_is_taken_as_no_root` | 默认 24 槽环：实例 1 写到 5，崩溃恢复抛弃 5（common 的现成造法），实例 2 写两次，实例 3 挂载那一刻读 5 那一槽报读错，覆盖写到 28 | 重读读出被抛弃的根：那一槽从根环表拿掉、不算知道住着根的槽；txg 28 那一版 I-3.1、I-3.11、I-2.1、I-4.8 不红（I-7.4 在这一形上今天就红，见第七节第 9 条，没判） |
| `the_instance_table_judging_the_rereads_after_the_mount_is_the_one_the_row_publish_wrote` | S = 4 小池写到 3 之后可写挂载 | 交回的分配器上判有效用的表 = 写行那次发布写下的那一张，≠ 挂载时最新那条根的那一张 |
| `the_warm_up_on_a_version_without_file_rereads_the_slot_unreadable_at_mount` | 树表 0 条的池接连挂载，14、23 两次写行崩在根槽前，下一次挂载写行 24、暖机 25，14 那一槽取号之前读错 | 暖机 25 那次零单元发布记根前重读了那一槽（挂载交回的根环表里它换成重读读出的有效根、算知道住着根的槽） |

RS3 在 `raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs`（盘面照正推腿探针、与已有那份用例同一做法另写一份）：

| 用例 | 钉住什么 |
|---|---|
| `admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount` | X = 21 一直读不出：准入抬 F 上限 21、F 抬到 21（改之前 22）；抬完 I-3.1、I-3.11、I-2.1、I-7.4、I-4.8 不红 |
| `admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x` | X 读得出：上限 21、F 21（对照组，不挂变异行） |

两条都不判 I-7.9：池级 checker 的 I-7.9 还按全环算上限，兄弟实现员 9-6 在改，正推腿探针上两种盘面今天都红。

## 三、第 4 条：`mount_transparency.rs` 改了哪几处

1. `use`：多 `DeviceIdentity`、`DeviceOffsetInBytes`、`SYSTEM_CONFIGURATION_SLOT_BYTES`。
2. `RefusalReasonOnTheD23List` 的类型注释与 `NewerPublishWitnessedBySystemConfiguration` 成员注释：「C331 取甲」换成「判不出改取 R2」，成员注释写清 R2 的判不出与 C554 乙-配置续 Q1 那一形。
3. 新加 `SystemConfigurationSlotsReadOnTheCrashImage` 与 `system_configuration_slots_read_on_the_crash_image`（每块盘两槽各直接读一次，自证过的与报读错的分开数；照第 8 批 8-2 那份改法，以今天主仓为底重做）。
4. 新加 `system_configuration_witness_undecidable_on_the_crash_image`（R2：有槽报读错且没有一块盘两槽都自证过）。
5. 新加 `some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`（第七节第 4 条）。
6. `newer_publish_witnessed_on_the_crash_image`：判不出（R2）或某块盘一份自证过的都没有 ⇒ 按真；否则 c_见证 取全部自证过的槽 journal tail 的最大值（撕裂的跳过），再走 `witnessed_counter_holds_against_the_selected_version`。文档注释跟着改。
7. C670 新判官 `witnessed_counter_holds_on_a_crash_image_without_read_faults` 的文档注释：「撕裂系统配置那一支…照甲把它判成按真，那一支的拒照 U6 判」那一句改了——改成「与改取 R2 之后在撕裂的轮换上同一个取法；两边只在某块盘两槽都撕裂那一形上分开，那一判按真、这里照常比」。它的代码一个字没改。

**C670 那句注释与断言（要转告形式化接入那条线）**：注释改了（上面第 7 条）；断言没改——`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的 `a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal` 照旧断言合法、照旧绿（那份文件不在我的清单里，我没碰）；它的文档注释还写着「U6 照甲把撕裂判成按真」，今天的判法里按真的理由换成了「这块盘不可见」那一项，措辞要那条线改。同一目录 `writable_mount_refusal_judged_against_the_d23_list.rs` 5 条、`writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 3 条在改完的判法上都绿（第六节原样输出）。

新用例在 `mount_transparency_judges_the_system_configuration_witness_by_r2.rs`：
- `a_witness_refusal_on_a_torn_system_configuration_rotation_is_a_refusal_for_the_wrong_reason`：两块盘世代 5 那一槽都撕裂、设备不报错；拼一份判据 N-配置 的拒，判 `Illegal { CriterionFalseWhenRecomputedOnTheCrashImage }`；取甲时判 `Legal`（变异行就是取甲）。
- `a_witness_refusal_on_a_device_without_any_self_verified_system_configuration_slot_is_legal`：盘 0 两槽都撕裂、盘 1 两槽都自证过，判 `Legal`。

## 四、第 5 条：清扫器

### 落点与形状

- 配置 `InstanceTableSweepConfiguration { switch, row_count_threshold, sweep_publish_placements }`：`switch` 两种（`SweptAtTheEndOfTheMountAboveTheThreshold` 开关关着、`HandedToTheBackgroundSweepWhileMounted` 开关开着）；`row_count_threshold` 是配置项，`PRODUCT` 取占位 `InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690 = 369`（一片实例表装得下的数据行数，越过它链长出第二片；不是定案，C690）；`sweep_publish_placements` 是只供测试的开关（第七节第 7 条）。产品路径 `mount_writable` 取 `PRODUCT`；今天每一条已有的流、历史与崩溃注入都在行数 ≤ 369 上，一个都不清扫，盘上写的与改之前逐字节相同（`a_sweep_on_a_version_without_file_is_not_supported_and_writes_nothing` 与 `the_product_configuration_does_not_sweep_a_table_at_or_below_the_threshold` 各钉了一形）。
- 判（`judge_the_instance_table_sweep`，只读）：池的盘数（系统配置 `device_count`）不全不删；根环每槽读一次，读不出或全零这一次不删，读得出而自证不过重读一次、仍不过这一次不删；候选行 = 现行那一版实例表里、实例在环里一条根都没有的行，没有就停（不扫池）；环里每条可读根的账（`allocation_records_under_root_in_the_unit_area_starting_at`）仍分配的槽记下、读不出这一次不删；每块盘从单元区起点按槽升序读 1024 字节认头（魔数、类、头校验和，不看载荷），认出就按整单元跳（码 1、码 3 两槽，码 2 一槽），读失败这一次不删，本池的单元写序实例大于挂载根的这一次不删；跳过账里仍分配的槽之后，写序实例是候选行、按那一行判未发布（码 1 看 b ≤ T ∨ n ≤ W，码 2 / 3 看 b ≤ T）的记进要抹的单子。
- 抹与发（`erase_the_heads_and_publish_the_sweep`）：每个要抹的头抹之前重读那一槽的头，仍是判的那一刻那个单元、仍按那一行判未发布才抹（FUA 写零头所在的那一个物理块）；全部返回之后每块盘一道屏障；再读一次现行那一版的实例表，去掉要删的行，`publish_version` 发清扫发布（同一实例、事务号 0、`InstanceTablePlan::Rewrite` 整链重写；没有普通分配，发布路径不判准入式子）。落点被拒（`PlacementRefused` / `SpaceAdmissionRefused`）报 `SweepPublishRefusedForSpace`、这一轮不删；挂载照样交回。
- 挂载末尾（`sweep_the_instance_table_at_the_end_of_the_mount`）：在 `establish_instance` 里删除续做之后、`Ok(Mounted)` 之前；发成了 `Mounted::current` 是清扫发布那一版。结局进 `MountOutput::instance_table_sweep`。
- 后台那一支：`sweep_the_instance_table_while_mounted` 是入口（判、抹前重读、发），判与抹发两半也各自公开（用例在两半之间插覆盖写）。挂着时什么时候调度后台那一轮没做：仓里没有后台调度器，这一批只给入口。

### 用例（`instance_table_sweep_at_the_end_of_a_writable_mount.rs`，S = 4 小环、128 MiB 内存盘）

盘面：实例 1 写到 5 → 十六个数据单元的顺序写崩在第一条记录写之前（孤儿）→ 实例 2 挂载、覆盖写到 19（实例 1 的根全轮出环）→ 实例 3 挂载。

| 用例 | 攻方哪一形 | 钉住什么 |
|---|---|---|
| `the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased` | 死实例删行 | 抹的落点 = 盘上留着的孤儿；孤儿头都认不出了；清扫发布那一版的表里只剩实例 2 的行；I-3.1 等五条不红 |
| `a_forged_head_inside_the_content_of_a_data_unit_is_claimed_by_the_unit_and_not_erased` | `sf-live-all`（用户内容里的伪头） | 实例 2 txg 8 那一版的内容在数据单元第二个槽开头带一个照抄孤儿的头；清扫时那一版出了环、单元已释放、第一个槽还是它自己的头：伪头不被抹、字节不变 |
| `a_head_inside_a_unit_still_allocated_in_the_account_of_a_ring_root_is_skipped_and_not_erased` | `sf-freed-skipalloc`（回退候选根引用的单元） | 同样的伪头在 txg 14 那一版，清扫时它的根还在环里；把那个数据单元第一个槽的头翻坏一个字节（认领规则在那一槽认不出单元）：伪头那一槽在环里那条根的账里，不被抹 |
| `a_read_failure_on_one_placement_deletes_no_row_and_erases_no_head` | 读失败不删 | 第一个孤儿那一槽每次读报设备错：结局 `AReadOfAPlacementFailed`、实例 1 那一行还在、孤儿头都在、挂载照样做成 |
| `the_background_sweep_with_a_device_missing_deletes_no_row` | 设备不全不删 | 开关开着挂载末尾不扫；后台那一轮只拿到盘 0：`NotEveryDeviceOfThePoolIsPresent { 1, 2 }` |
| `the_background_sweep_rereads_each_head_before_erasing_it_and_leaves_a_slot_allocated_after_the_judgement` | `ss-7-stale`（判完到抹之间有分配） | 后台先判，再覆盖写到单子上一个落点的头被盖掉（至多 128 次），再抹、发：那一槽不抹，别的照抹，实例 1 那一行照删，现行那一版读得回来、五条不红 |
| `a_sweep_publish_refused_for_its_placement_deletes_no_row_and_the_mount_stays_writable` | 清扫发布被落点拒 | 只供测试的落点开关：`SweepPublishRefusedForSpace`、实例 1 那一行还在；接着覆盖写一次做成 |
| `the_product_configuration_does_not_sweep_a_table_at_or_below_the_threshold` | 阈值 | `PRODUCT` 两行不扫，孤儿头都在 |
| `a_sweep_on_a_version_without_file_is_not_supported_and_writes_nothing` | 第一版不支持的那一格（第七节第 6 条） | 树表 0 条的池第八次挂载有一行就扫：结局是那个成员；盘面与不扫的同一次挂载逐字节相同、录制流一样长 |

core 库单测 3 条（`instance_table_sweep.rs` 里）：已发布谓词两支、认领跨度、全零与魔数不对的槽不是单元。

### checker 档（只编、没跑）

`crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs`（第一行 `//! checker 档模块：crash、layer0_progress、crash_injection`）：同一类盘面（两个孤儿数据单元），录实例 3 那次清扫挂载的全部写，基线是挂载之前的盘面。

- `quick_tier_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row`（快档，不标 ignore）：甲二展开，每个状态恢复、oracle、池级 checker 与 F5；F5 = 恢复落到清扫发布那一版的状态里，按删前那一行判未发布、写序实例是那个实例的本池单元一个都不许剩（这份用例另写一份认头与认领，不调清扫器）；判到的删行生效的状态数要 > 0。
- `every_crash_state_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row`（`#[ignore]`）：全量展开，判法同上。
- `every_crash_state_inside_the_two_sweep_phases_remounts_writable_and_ends_without_the_deleted_row`（`#[ignore]`）：只展开落在抹头与清扫发布两段（`crash_injection::writes_of_the_instance_table_sweep_phases` 划段）的段，每个状态再起一次有一行就扫的可写挂载：挂载做成、五条不红、之后实例 1 那一行不在、F5 一个不剩；多线程分片。

比已有的流多罩的：已有层 0 流与崩溃注入里没有一次挂载写过抹头与清扫发布；这一条的挂载末尾两段每个崩溃态都枚举，第三条多跑一步再挂。崩溃注入战役本身仍用 `PRODUCT`（不清扫），新加的两段在战役里摆不到写，只在这份用例与那条库单测里走到。C597 判别力：「不抹就删」那一行（`第9批 清扫 C597 判别力 F5「不抹就删」…`）点名快档那一条，主 agent 提交前跑。

## 五、证红：改坏哪一处 → 哪条断言红

全部用 `bash scripts/prove-red.sh --copy /tmp/claude-1000/m3-r1/batch9-core2/work --memory 8G singlefs-harness <名…>` 跑（行在副本的 `crates/mutations.tsv` 里，与 `mutations-append.tsv` 同一份）。每行的 cargo 参数都带 `-- <那条用例名>`，一行只跑那一条用例，同时红了别的用例这一趟看不到。两趟的汇总行原样：第一趟（5 行）「✓ 点名 5 条：跑了 5 条，跳过 0 条，跑的都抓到了」；第二趟（15 行）「✗ 点名 15 条：跑了 15 条，跳过 0 条，有没红或无效的（逐条列在上面）」——没红的那一行是 q3，用例与变异行之后都删了（第七节第 2 条），其余 14 行都抓到了。日志在 `/tmp/claude-1000/m3-r1/batch9-core2/prove-logs-2/`、`prove-logs-3/`（`baseline.log` 加逐行编号）。

| 变异（改坏哪一处） | 红的用例 → 断言消息原样 |
|---|---|
| `mount.rs` 重读分类：读得出而自证不过改回当没有根 | `a_valid_root_read_bad_once_on_the_reread_is_counted_at_the_highest_txg_its_slot_can_hold` →「重读读到自证不过：照旧按能住的最大 txg 18 记着」 |
| 同上（第二行，点名前洞那一条） | `a_front_hole_unreadable_at_mount_counts_more_allocated_only_on_the_version_that_rereads_it` →「txg 20：重读读到自证不过（从没写过），照旧按 9 记着」 |
| `allocator.rs` `ring_slots_known_to_hold_a_root` 不收重读读出的有效根 | `a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write` →「前提：重读读出了有效根」（红在前提那一断言上：那一槽不算知道住着根的槽） |
| `mount.rs` 重读分类「读得出有效」那一臂加 `false &&`（p1） | `a_valid_root_read_on_the_reread_inside_the_window_holds_the_reclaim_threshold_at_its_txg` →「重读读出有效根：不再按挂载时读不出记，那一槽算这个进程知道住着根的槽」 |
| 同一臂加 `true \|\|`（p2：不按表判有效） | `an_abandoned_root_read_on_the_reread_is_taken_as_no_root` →「重读读出被抛弃的根：当没有根，从根环表拿掉，不算知道住着根的槽」 |
| `mount.rs` `establish_instance` 不换判有效用的表 | `the_instance_table_judging_the_rereads_after_the_mount_is_the_one_the_row_publish_wrote` →「判有效用写行那一张」 |
| `mount.rs` `publish_empty_after` 树表 0 条那一支记根不重读（q1） | `the_warm_up_on_a_version_without_file_rereads_the_slot_unreadable_at_mount` →「暖机那次记根之前重读了那一槽」 |
| `mount.rs` 上限不按挂载时读不出的槽封顶（RS3） | `admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount` →「上限与抬到的 F 都不越过 X 那一槽能住的 txg 21」 |
| `mount_transparency.rs` 判不出回到取甲 | `a_witness_refusal_on_a_torn_system_configuration_rotation_is_a_refusal_for_the_wrong_reason` →「撕裂的轮换不是判不出：判据复算为假」 |
| `mount_transparency.rs` 不认某块盘一份自证过的都没有 | `a_witness_refusal_on_a_device_without_any_self_verified_system_configuration_slot_is_legal` →「一块盘一份自证过的系统配置都没有：判据按真」 |
| `instance_table_sweep.rs` 抹头那一写换成不写（不抹就删） | `the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased` →「孤儿 PlacementOfAUnitHead { device: DeviceIdentity(0), slot: SlotNumber(1114) } 的头抹掉了」 |
| 认出单元之后只跳一槽 | `a_forged_head_inside_the_content_of_a_data_unit_is_claimed_by_the_unit_and_not_erased` →「伪头所在的槽不在抹了的落点里」 |
| 不跳环里可读根的账 | `a_head_inside_a_unit_still_allocated_in_the_account_of_a_ring_root_is_skipped_and_not_erased` →「伪头所在的槽在环里那条根的账里，不在抹了的落点里」 |
| 读失败当不是单元 | `a_read_failure_on_one_placement_deletes_no_row_and_erases_no_head` →「读失败这一轮不删：Swept(RowsDeleted { … })」 |
| 不判设备全不全 | `the_background_sweep_with_a_device_missing_deletes_no_row` →「设备不全不删：NoRowDeleted(ARootRingSlotIsUnreadableOrAllZero { ring_slot: RootRingSlot { region: 1, slot: 0 } })」 |
| 抹前不重读 | `the_background_sweep_rereads_each_head_before_erasing_it_and_leaves_a_slot_allocated_after_the_judgement` →「被现行那一版占了的那一槽没抹」 |
| 落点被拒不归不删那一格 | `a_sweep_publish_refused_for_its_placement_deletes_no_row_and_the_mount_stays_writable` →「清扫发布被落点拒：Swept(SweepPublishFailed { … })」 |
| 开关关着时不看阈值 | `the_product_configuration_does_not_sweep_a_table_at_or_below_the_threshold` →「没超过阈值不扫：Swept(RowsDeleted { … })」 |
| 树表 0 条那一版报成没有候选行 | `a_sweep_on_a_version_without_file_is_not_supported_and_writes_nothing` →「树表 0 条的一版上不清扫：Swept(NoRowDeleted(NoCandidateRow))」 |

没证、留给提交时的 checker-tier-crates-mutation-replay（checker 档包，派发说只编不跑）：`第9批 清扫 C597 判别力 F5「不抹就删」…`（点名 `crash_points_of_the_instance_table_sweep` 的快档那一条）、`第9批 清扫两段：抹头之后不按 FUA 分清扫发布…`、`第9批 清扫两段：WritableMountPhase::ALL 里没有抹头与清扫发布…`（后两行点名 `crash_injection::tests` 新加的那一条库单测）。替换的 5 行也没证（名字照旧，替换文跟上新代码）。

core 库单测 3 条（`instance_table_sweep.rs` 的 `tests`）没有单独的变异行：它们钉的谓词与跨度，已由上表「只跳一槽」「抹头不写」两行在 harness 档那一侧抓到；没挑一行给它们自己证红——这是缺口，照实列。

后来删掉的一条：`the_row_publish_on_a_version_without_file_rereads_the_slot_unreadable_at_mount`（q3，树表 0 条那一版写行那一处记根换成不重读）——变异跑出来「没红」，原因在第七节第 2 条；用例与那一行一起删了，不留一条证不了会红的用例。

## 六、交回前的验证（副本里，末尾原样输出）

一条 Bash 里串着跑，各段退出码各写一份（`final-*.exit`）；harness 测试与 core 库单测那两段第一次被内存包装按峰值表拒了（退 251，命令一行都没跑），换上 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 单独再跑了一次（第十节）。

`cargo fmt --all -- --check`：没有输出，退出码 0。

`cargo clippy --offline --keep-going --all-targets --all-features -- -D warnings` 加 `check.sh` 的 `CODE_DISCIPLINE_LINTS` 七条（`BINDGEN_EXTRA_CLANG_ARGS` 取 `crash-amplification-lib.sh` 那一份），退出码 101；编不过的编译单元原样：
```
error: could not compile `singlefs-checker-tier` (bin "e171_three_oracle_collusion") due to 2 previous errors
error: could not compile `singlefs-checker-tier` (bin "e165_replay_bounded_enumeration") due to 2 previous errors
error: could not compile `singlefs-checker-tier` (bin "e171_three_oracle_collusion" test) due to 9 previous errors
error: could not compile `singlefs-harness` (test "an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping") due to 2 previous errors
error: could not compile `singlefs-checker-tier` (bin "e165_replay_bounded_enumeration" test) due to 7 previous errors
```
告警落在哪些文件（`-->` 行按文件数）：
```
     14 crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
     18 crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
      4 crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
```
三份都不是这一轮改的、都在开工副本里原样（`shadow_unrelated` 一类）；我改的与新建的文件 0 条告警（第一趟 clippy 报过我四份用例文件里的同名遮蔽，已改名，这一趟不在了）。

`cargo build --offline --all-targets` 末尾原样（退出码 0）：
```
   Compiling singlefs-checker-tier v0.1.0 (/tmp/claude-1000/m3-r1/batch9-core2/work/crates/singlefs-checker-tier)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.93s
```

harness 档动到的测试二进制（`--no-fail-fast`，八个二进制：四个新的、四个判法或行为被这一轮改到的已有用例）末尾原样（退出码 0）：
```
     Running tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs (target/debug/deps/instance_table_sweep_at_the_end_of_a_writable_mount-8b831e6ef754184b)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
     Running tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs (target/debug/deps/mount_transparency_judges_the_system_configuration_witness_by_r2-17ae37e919385c4b)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs (target/debug/deps/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window-8ea9accc97fe5292)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.69s
     Running tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs (target/debug/deps/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount-8261bbb42c293d77)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.16s
     Running tests/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs (target/debug/deps/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable-97fb7193859383c4)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.57s
     Running tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs (target/debug/deps/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window-035136d7909d5b63)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.89s
     Running tests/writable_mount_refusal_judged_against_the_d23_list.rs (target/debug/deps/writable_mount_refusal_judged_against_the_d23_list-aeffd3de06aaa3bb)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs (target/debug/deps/writable_mount_refusal_on_a_crash_state_without_read_faults-570f5697dea4fee3)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```
`cargo test -p singlefs-core --lib` 末尾原样（退出码 0）：
```
test result: ok. 183 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

`scripts/crash-case-check.py` 在副本上：file-names「✓ 查了 176 份测试文件的名字…」、one-scenario「✓ harness 档 984 条用例都是一条一个场景」；placement 报我那两条 `#[ignore]` 没登记（第八节给登记行）；modules 报的那一处是别人的 `gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs`，不是这一轮的。

## 七、停下交主 agent 的、我取了读法的、做不出会红用例的

1. **`mount.rs` 写行零单元那一处记根（`establish_instance` 里 `rows_written` 为空那一支，探针 q2）走不到重读**：那一支只在取号 1（第一次可写挂载）时走；那一刻根环里只有 mkfs 写的 txg 0，挂载时记成读不出的槽只能落在区间 (N − 3S, first_txg − 1] = [0, 0] 里、即 txg 0 那一槽，而它读不出时择不到根、挂载不成（`unreadable_ring_slots_inside_the_window` 的区间，`mount.rs` 那个函数里 `lowest_txg_inside_the_window` 与 `tail_of_the_interval` 两行）。暖机紧接着也一样。没写用例、没挂变异行；调用照旧留着（没删，不在条款里的改动不做）。
2. **`transaction.rs` 写行那一处（`prepare_the_row_publish_on_a_version_without_file`，探针 q3）做不出会红的用例**：写行那次记根时会被重读的那一槽，按构造是窗口里最旧的 txg = 写行 txg − 3S + 1 那一槽，也就是写行 txg + 1 那一槽；两块盘的池上每次可写挂载在写行之后至少推一次暖机（`warm_up_publish_txgs`：写行那一槽与下一槽总有一块盘没覆盖），暖机那一次就写这一槽，根环表那一项被这条根盖掉、回收在同一次补上；那一版又没有记账树（树表 0 条），落点在记根之前就取完了。我造了「洞 14、洞 24，写行 25 那次重读读出 txg 2 的旧根」那一形，变异照跑、没红，用例与变异行都删了。要钉它，只有单盘池（三区都在一块盘上、不推暖机）这一形，单盘能不能可写挂载要 D2 的 w 下限，没有现查，交主 agent 定要不要另做。
3. **「写行之前用最新根那一张」（`rebuilt_allocator` 里 `note_the_instance_table_judging_the_rereads_of_root_ring_slots(newest_table.clone())`）没有消费者**：`rebuilt_allocator` 只有可写挂载一个调用点，装上之后到 `establish_instance` 换表之间，只有取号之前在分配器拷贝上的预演记根，预演走不重读的 `record_root_written_by_this_process`；所以那一张表从来不被读。改它的变异是等价变异，没挂。「写行之后换写行那一张」那一半有用例与变异行。那一行赋值留着（D13 / 代码纪律「用不到的代码删掉」要不要删，交主 agent）。
4. **R2 判法里「某块盘一份自证过的系统配置都没有 ⇒ 按真」**：R2 的判不出本身（D22 已定项 8）不罩这一形；实现在读阶段判完之后单列 `WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`、报成判据 N-配置 的拒因、`witnesses_a_publish_newer_than_the_selected_version` 按真（C554 乙-配置续 Q1，`mount.rs` 那个成员的文档注释）。harness 判法照同一个处置复算；不这样做，已有的 `a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal` 会判成拒错了。这是照实现那个成员已写的处置，不是新定的；C664（两种拒因报成一个）仍开着。
5. **RS3 只并进「挂载那一刻读不出」的槽**：D16「根槽这一次读坏」那一行写的是「挂载那一刻就读不出**或自证不过**、这个进程之后也没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根」，「抬 F 的上限」那一行只写「读不出」。实现里挂载时自证不过的槽不进根环表、不知道它能住哪个 txg（从没写过的槽、撕裂的槽都在这一类），我照「抬 F 的上限」那一行只封顶根环表里记着的读不出的槽。两行字面不一样，要不要把挂载时自证不过的槽也并进来、按哪个 txg 并，交主 agent。
6. **清扫在树表 0 条的一版上第一版不做**（`NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion`，在任何写之前、一个头都不抹）：那一版的账住根记录那一项、不在树表里（C512），「跳过根环里可读根的账」那一半用 `allocation_records_under_root…` 读不全；清扫发布在那一版上的形状（走写行那一路的 `publish_instance_table_on_version_without_file` 还是别的）条款没写。用例钉住「报这个成员、盘面与不扫的那一次逐字节相同」。
7. **清扫的块设备错条款没写**：抹头的一个写或屏障报错 → `StoppedByABlockDeviceErrorWhileErasing`，清扫发布落盘途中报错或因落点之外的理由被拒 → `SweepPublishFailed`；两种都这一轮不删、挂载照样交回（照 Q4「被拒就这一轮不删、挂载照样交回可写」那一格的处置外推：已抹的头按那一行本来就未发布）；落盘途中报错那一次照发布路径冻结在分配器上。没有用例走到这两支（要造块设备错的包装），是缺口。
8. **清扫自取的两格**：根环里一条可读根的账读不出 → 这一轮不删（`AccountOfARingRootUnreadable`）；池里有本池单元的写序实例大于挂载根的 → 这一轮不删（判决第二节 Q3 正推给的保守读法）。只供测试的落点开关 `SweepPublishPlacements::EveryFreeSlotTakenAwayByTheTestOnlySwitch` 在分配器拷贝上把空闲槽全占掉、走真的落点那一段（拷贝上取到落点就 panic，说明开关没把那一支强制进去）。
9. **一形已知红、不归这一轮**：被抛弃的根所在的槽挂载那一刻读不出时，影子账罩不到它引用的单元，之后被合法复用，池级 checker 判 I-7.4 红（攻方 K1 第 3 小节「被抛弃」那一条已写：病根在挂载那一刻影子账读不到那一槽，与 E2 无关）。`an_abandoned_root_read_on_the_reread_is_taken_as_no_root` 因此不判 I-7.4，用例注释里写着。
10. **挂载透明不认清扫发布**：`mount_transparency.rs` 的逐字段判法里实例表行那一样按「恢复写的行」比，清扫发布删了行就会红；崩溃注入战役今天用 `PRODUCT`（不清扫），所以不碰。以后战役要带清扫的配置，判法要先跟上。
11. **用例放的文件**：RS2 那几条放进了派发给的 `root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs`，文件名只说了 RS1 那一格；要改名交主 agent。

## 八、要登记进 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行（原文）

```
crash-case:instance-table-sweep-mount	crates/ Cargo.toml Cargo.lock	test=singlefs-checker-tier:crash_points_of_the_instance_table_sweep:every_crash_state_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED	# 实例表清扫那次可写挂载（抹头、清扫发布）的全部写按层 0 全量展开，每个状态恢复、oracle、池级 checker 与 F5（删行那一版生效的崩溃态里被删实例按删前那一行判未发布的单元一个不剩，C597 的验收）
crash-case:instance-table-sweep-phases-remount	crates/ Cargo.toml Cargo.lock	test=singlefs-checker-tier:crash_points_of_the_instance_table_sweep:every_crash_state_inside_the_two_sweep_phases_remounts_writable_and_ends_without_the_deleted_row count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED	# 清扫两段（抹头、清扫发布）上的每个崩溃态再起一次可写挂载（有一行就扫）：挂载做成、池级 checker 不红、那一行删掉、F5 一个不剩
```

两条都只跑一次 `enumerate_layer0_in_state_slices`，`LAYER0_PARALLEL_FINISHED` 那一行恰好一行；没打 `exhaustive=`（不是两条层 0 主流）。状态数没算（没跑）。

## 九、要主 agent 跑的命令（都在提交时那一趟里，checker 档，我没跑）

工作目录：打上补丁之后的主工作区根（或暂存树）；环境变量前缀 `SINGLEFS_HEAVY_TESTS=commit`；内存上限与线程数照 `.claude/skills/crash-test/commit-stages.md`。

1. checker 档快档里这一轮新加的两样（随 `checker-tier-layer0-replay` 快档那一趟本来就跑，单点名时）：
   `cargo test --release -p singlefs-checker-tier --test crash_points_of_the_instance_table_sweep`（判：`quick_tier_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row … ok`、两条 ignored）；
   `cargo test --release -p singlefs-checker-tier --lib -- crash_injection::tests::the_writes_of_the_instance_table_sweep_split_into_erasing_heads_and_the_sweep_publish_and_each_gets_a_second_crash_point`（判：那一条 ok）；
   崩溃注入战役的已有快档用例照跑（`WritableMountPhase::ALL` 从 4 段变 6 段；战役用 `PRODUCT` 不清扫，新的两段在战役里摆不到写，推的、没跑）。
2. 第八节两行登记之后的全量：`checker-tier-layer0-replay --full --item instance-table-sweep-mount --item instance-table-sweep-phases-remount`（判：各自的 `LAYER0_PARALLEL_FINISHED` 行与用例 ok）。
3. `checker-tier-crates-mutation-replay` 复跑追加的 22 行与替换的 5 行，其中我没证过的是 checker 档那 3 行（第五节）与替换的 5 行。

## 十、cargo 调用与负载

开跑前 `ps` 看到别的会话在跑 `cargo test`（`/tmp/claude-1000/m3-formal-code-b19-r1-sonnet/ws` 的 `b19_sonnet_probe`）、主 agent 的看门狗；没有 qemu / vm-bench / e152 / fio。没等锁。编译与测试的 cargo 调用按次数：`cargo build --all-targets` 两次、harness 测试两次（第一次没带 `--no-fail-fast`，红在一条用例上就停了）、`cargo clippy` 两次（第一次没带 `--keep-going`，停在别人的文件上）、`prove-red.sh` 三次（第一次每行都被内存包装按峰值表拒、只跑了基线；后两次 5 行、15 行）、末尾那一条串的命令一次——到这里是 10 次；末尾那一条里 harness 测试与 core 库单测两段被包装拒、一行没跑，我又单独跑了一次这两段（第 11 次起 cargo，只跑不编），为的是拿到这两样的末尾原样输出。线程上限 10 一律经 `capped.sh 10`。

## 十一、没做什么

- 没走三方对抗；checker 档（`checker-tier-layer0-replay` 快档与全量、QEMU、herd7、crates 变异整表、形式化验证）一样都没跑，归提交时的主 agent；没提交。
- checker 档那 3 行变异、替换的 5 行变异、core 库单测 3 条没证红（第五节）。
- 清扫器后台那一支只给了入口，挂着时的调度没做；块设备错那两支没有用例（第七节第 7 条）。
- 没跑整份 `cargo test -p singlefs-harness`：只跑了第六节那八个二进制；别的用例经 `mount_writable` 的都多了 `MountOutput::instance_table_sweep` 与交回前那一判，`PRODUCT` 配置下行数 ≤ 369 一律不扫（不读盘、不写盘，只读一次现行那一版的实例表数行数），推的没有行为变化，没有现跑证明。
- `.claude/gate.d/stage-inputs.tsv` 的登记、kb 与 `checks-owed.md`（C597、C690、C333 题面）不在我的写范围，第八节给了登记行原文。

## 十二、草稿目录

`/tmp/claude-1000/m3-r1/batch9-core2/`：`work/`（仓副本连它自己的 `target/`）交回之前删掉；留下的只有日志与这份补丁目录（`patch/`、`prove-logs-*`、`*.log`、`rows.py`、`anchor_check.py`）。
