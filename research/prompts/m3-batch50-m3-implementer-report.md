# 第 50 批实现员报告：退役树（R1–R3）、M3 合并与均分借、准入两样都做、checker 五条

日期：2026-10-09（Asia/Tokyo）。批次 `records/2026-10-01-删除与并发事务.md#第 50 批`。草稿目录 `~/.local/share/singlefs/work/m3-batch50/m3/`。

## 一、结论

- 八条都写了代码；harness 档动到的 184 个测试二进制全部跑过：182 个全绿，2 个红（11 条，全在「不补环窗口的窄盘」那一族造形上，见第七节），这 11 条没修好，停在编得过的点交回。
- 新加的测试都绿；**证红一条都没跑**：prove-red 在副本里每条要重编、超过前台 240 秒，变异行已写好（追加 37、替换 36、删 2），证红命令在第十二节交主 agent 跑。
- 本派发 cargo 编译与测试调用用满 10 次（见第六节），`cargo build --offline --all-targets` 与派发点名的 `cargo build -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes` **没跑**（预算用满）；最后一次 `cargo clippy --all-targets --all-features` 退 0，它把 e171 那个 bin 一起做过类型检查。e171 源文件与主工作区逐字节相同（`cmp` 过）。
- 设计问题三条要主 agent 定（第十节），其中第 1 条（GPU 只判 I-9.31、其余回落 CPU，与「配了 GPU 就一定要 GPU 判」和 GPU 对拍「一个状态都不许判不了」相撞）会让 checker 档的 GPU 对拍与带 GPU 的崩溃放量在含删到底的流上红或报错。

## 二、补丁

两套，格式照 `scripts/apply-writer-patch.py` 文件头（`mutations-delete.txt` 与派发写的 `mutations-deletions.tsv` 内容相同，两份都放）：

1. 打在「主工作区 + W3 补丁（`~/.local/share/singlefs/work/m3-batch49/w3c/patch/`）」上：`~/.local/share/singlefs/work/m3-batch50/m3/patch/`
   - `crates.patch`：62 个文件，4073 行加、645 行删。
   - `mutations-append.tsv` 37 行、`mutations-replacements.tsv` 36 行、`mutations-delete.txt` 2 行。
2. 打在主工作区现在这一版上、W3 与本批合起来：`~/.local/share/singlefs/work/m3-batch50/m3/patch-combined/`
   - `crates.patch`：70 个文件，6541 行加、873 行删（主工作区拷出的副本 `base/` → 本批之后的副本）。
   - `mutations-append.tsv` 66 行（W3 的 29 行 + 本批 37 行；本批要替换、删的名字没有一个落在 W3 追加的行上）、`mutations-replacements.tsv` 44 行（W3 的 9 行与本批 36 行合并，重名的取本批的）、`mutations-delete.txt` 2 行。
   - 合并时核过：替换与删的名字在主工作区现在的 `crates/mutations.tsv` 里各恰好一行，追加的名字一个都不在表里（主工作区的表与 `base/` 那份逐名相同，3520 行）。

## 三、这一轮写过的文件（我自己列）

在 W3 补丁那 33 份之上本批动过的（补丁里 62 个文件全部在下面）：

- 格式与核心：`crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/{admission.rs, container_tree.rs, inode_tree.rs, lib.rs, mount.rs, records.rs, recovery.rs, retirement_tree.rs（新建）, transaction.rs, unit.rs, unlink.rs, write_accounting.rs}`
- 池级 checker：`crates/singlefs-checker/src/{image.rs, lib.rs, walk.rs}`
- checker 档：`crates/singlefs-checker-tier/src/{crash_injection.rs, crash_judge_checker.wgsl, crash_judge_checker_common.wgsl, crash_judge_checker_scan.wgsl, crash_judge_common.wgsl, crash_judge_gpu.rs}`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`
- harness 库：`crates/singlefs-harness/src/{history.rs, model.rs, model_comparison.rs, mount_transparency.rs, scenario.rs}`
- harness 测试（`crates/singlefs-harness/tests/` 下，钉值与新用例）：`a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs`、`accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf.rs`、`admission_checkpoint_cost_per_device_paths.rs`、`admission_formula_on_the_mounted_state.rs`、`admission_raises_the_floor_before_refusing.rs`、`admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs`、`admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs`、`checker_known_bad_images.rs`、`checker_narrow_invariants_and_abandoned_roots.rs`、`common_admission_pool/mod.rs`、`common_deletion_in_progress/mod.rs`、`container_trees_grow_a_level_when_the_root_is_full.rs`、`corrupt_on_disk_content_is_refused_instead_of_panicking.rs`、`deletion_scenario_writes.rs`、`inode_leaf_absorb_and_borrow_on_the_unlink_path.rs`（新建）、`mount_transparency_judged_field_by_field_after_a_writable_mount.rs`、`new_pool_file_creation_publish.rs`、`overwrite_in_one_instance.rs`、`pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`、`pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs`、`rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs`、`rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs`、`rollback_by_a_forward_publish.rs`、`rollback_lineage_record_and_its_checkpoint_cost_term.rs`、`the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs`、`unequal_devices.rs`、`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`、`unlink_crash_image_remounted_twice_continues_to_identical_disks.rs`、`unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs`、`unlink_mount_continuation_and_the_emptied_version.rs`、`unlink_single_file_publishes.rs`、`unlink_writes_per_transaction_do_not_grow_with_the_file.rs`、`writable_mount_of_a_formatted_pool.rs`、`write_accounting_per_structure.rs`
- `crates/mutations.tsv`：删 2、替换 36、追加 37（名字见第九节）。

清单外动了的，逐个写为什么：

| 文件 | 为什么 |
|---|---|
| `crates/singlefs-checker/src/lib.rs` | checker 自己那份 key 形态登记要认种类 13（`10 \| 11 \| 13` 走容器树条目形态），不改退役树根条目判不了 |
| `crates/singlefs-checker-tier/src/crash_injection.rs` | 它对 `WrittenStructureKind` / `TransactionUnit` 穷举 match，新成员不补上编不过（只加了退役那几条臂） |
| `crates/singlefs-core/src/write_accounting.rs` | 写账多一种 `RetirementTreeNode`，`IN_REPORT_ORDER` 加一格（形式化线第 3497 行变异的锚点照旧恰好命中一次，核过） |
| `crates/singlefs-harness/src/history.rs` | 小盘为环窗口补足的槽数 815 / 739 → 1007（式子写在常量注释里） |
| `crates/singlefs-harness/src/mount_transparency.rs` | 「只有续做会改根」的种类集合加退役树（续做的最后一批写退役树根），5 → 6 种 |
| `crates/singlefs-harness/src/scenario.rs` | `DeletionSwitches` 多一个字段（退役容器容量开关），字面量补上 |
| `crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs`（新建） | M3 吸收、借、借后连锁吸收在发布路径上的三条用例，放进已有的文件里都不对题 |
| `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`、`common_admission_pool/mod.rs` | 测试共用模块：前者加改退役记录的几个帮手，后者同步补足常量 1007 |

清单里列了而没动的：`tombstone_tree.rs`、`make_filesystem.rs`、`deleted_inodes.rs`、`crash_judge_tables.rs`（容器树共用的一半在 `container_tree.rs`；mkfs 写的第 0 版树表是空的；GPU 树表容量在 `crash_judge_checker_common.wgsl` 的 `C_MAX_TREES_PER_TABLE`）。

`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 没动；它引的公开名字与签名没改，它穷举 match 的 `MountError`、`RefusalReasonOnTheD23List`、`RefusalConditionNotMet` 没加成员。

## 四、git diff --stat

主工作区不动；补丁在草稿目录的临时 git 仓里出（`git diff --cached --stat -- crates litmus` 末行原样）：
- 打在「主工作区 + W3」上：`62 files changed, 4073 insertions(+), 645 deletions(-)`
- 打在主工作区上、合起来：`70 files changed, 6541 insertions(+), 873 deletions(-)`

## 五、各条做了什么、取的是什么

1. **R1 登记**：`TREE_KIND_RETIREMENT = 13`（`records.rs`），`TREE_IDENTIFIER_RETIREMENT = 65548`、`TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH = 65549`、`NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES = 12`；第一个文件版本那次零根注册、排在 logged_ops 之后，头 ID 写 inode 树的树 ID 65537；树表声明长度 2400。`tree_table_holds_twelve_entries_keyed_by_tree_identifier`（原 eleven）逐字段钉 12 条；mkfs 65536、第一个文件那次 65549 钉在格式单测与 `writable_mount_of_a_formatted_pool.rs`。
2. **退役记录与退役树**：`RetirementRecord`（34 字节，身份四段与 `PackedIdentity` 同序同宽，死亡 txg 末 8 字节）、`RETIREMENT_RECORD_BYTES = 34`、`RETIREMENT_CONTAINER_RECORDS = 959`（带 `format-const:` 标记）；打包记录类型 3。形态照墓碑树、共用 `container_tree.rs`：「一次追加一条」推广成「一次追加多条」（`append_records_to_the_open_container`），重写的码 2 节点 = 被碰到的每片容器的祖先并集，长高那次旧根照抄的判断抽成 `previous_root_is_carried_down_unchanged`。回退照抄现行那一版（`mount.rs`）；崩溃恢复随所选根（`recovery.rs` 按树表读回）。单测见第十二节。
3. **死亡写侧**：`remove_record_from_leaf_containers` 交回 `retired`（身份消失的那几片，按上一版叶序）；`transaction.rs` 同一次发布追加，死亡 txg = 这次的 checkpoint_txg；借不写；不去重不覆盖；条数不设上限。用例：删空 `the_final_batch_that_empties_the_only_leaf_writes_one_retirement_record_and_every_retirement_invariant_holds`、吸收 `the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one`、借后退役树不变 `a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing`、「删 → 回退 → 再删」两条 `unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens`。
4. **M3**：照 E176 M3 那一臂：删空摘掉；先查右邻、再查左邻，两片之和 ≤ 233 左吸收右（保留左半身份与分隔 key），并完接着查；一次没并、少于 117 条且有右邻就借 ⌊(右 − 左) ÷ 2⌋ 条最小的（右片保留身份，分隔 key 改成它这一版最小 key，N1），借完从借出那片接着看合并。`InodeLeafContainer` 多字段 `separator_key_in_the_parent_entry`。`inode_tree.rs` 文件头那句改了。模型（`model.rs`）与胶水（`model_comparison.rs`）跟上，随机历史对拍全绿。我取的是「借完从借出那片看合并」（E176 实验页与 `e176_inode_leaf_merge_capacity_and_rewrites.rs` 的 M3 臂），发布路径上走到了：`the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it`。
5. **R3**：`InodeLeafContainerOrigin::{RewrittenThisPublish, CarriedFrom(上一版叶序)}`：挪了位置而内容没变的片照抄上一版落点、字节与映射 key，只换角色名，不进重写清单；身份消失的片由上一版叶序的角色释放。为什么：盘上按身份引用找孩子，角色序号只在内存里。单测 `dropping_the_emptied_leftmost_container_carries_every_container_to_its_right_whatever_their_number` 在 20 片与 40 片上钉重写清单空；改回按位置重写留了变异行。发布路径上走不到见设计问题第 2 条。
6. **准入两样都做**：ckpt_cost 加 2 + 1 + H_退役，中央映射树删插数加 2 + H_退役 把；标签 6 加 (2 + H_退役 + 1) × 16384 + 35 × n_记录（35 = ⌈32768 ÷ 959⌉，n_记录 含 nlink = 0 的）；W 加 6 + H_退役（H 从要接着那一版的根码 2 头现读）；退役三种角色在 `space_budget_of_role` 归普通分配。绝对值钉在 `admission.rs` 单测与 harness。贴边那条用例没写成，见设计问题第 3 条。
7. **checker**：删 I-9.4 第三句（CPU 与 GPU），点名它的两行变异删了；原来证它红的坏镜像改成合法形状的阳性对照 `a_borrowed_right_container_numbered_below_the_largest_key_on_its_left_holds_every_invariant`，留一行「第三句判回来就红」的变异。I-9.12 判法是「分隔 key 落在孩子区间之外」，N1 之后不用改，没改。I-9.16 ④ 11 → 12 种；I-9.27 加种类 13。新立 I-9.31–I-9.35（`IMPLEMENTED_INVARIANTS` 85 → 90）：I-9.31 退役树条目身份与子头相符（类型段 {0, 3}）；I-9.32 每条：被退役身份类型 == 2、出生树 ≠ 0、容器出生代 < 死亡 txg ≤ 本版 txg，每片 ≤ 959 条，不滤谱系；I-9.33 父链上的按身份不重复；I-9.34 父链上的不指本版活叶（读内部条目身份引用）；I-9.35 父版（谱系树里 T_rb = 本版 txg 取 T_old，否则 txg − 1；不在候选集不判）叶身份集 − 本版 = 本版死亡 txg == 本版 txg 的身份集，漏记、多记都报；候选集滤不掉被抛弃根时整条报部分没判；退役树头 ID 配不上 inode 树时那一版报判不了。坏镜像（登记进新表 `RETIREMENT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE`）各钉「只红它自己」：I-9.31、I-9.32、I-9.33、I-9.34、I-9.35 漏写与多写、I-9.27 种类 13 头 ID 写 0。合法镜像不红：删到底、删 → 回退 → 再删、回退越过一片新叶的出生（`rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens`）、吸收、借、借后吸收。GPU：I-9.31 同判（`INVARIANT_I_9_31 = 85u`）；候选集里有一版退役树不是零根时扫描内核记码 8、交 CPU；树表容量 11 → 12，草稿区末端 317104 字。
8. **钉值**：标签 6 = 2 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)：第一个文件那一版 196760、意图到最后一批之前 196678、删完 196608、回退回来 196760；W：4 GiB 374（25 × 374 = 9350 槽）、16 GiB 386、批大小 1 时 88、384 槽小盘 36；ckpt 多扣块数（空发布不写退役树，各多 3）：1 GiB 138–140、4 GiB 533–535、64 GiB 10343–10345、1 TiB 167304–167306、两层映射 534–536，压小映射树三格现跑 885–891、1067–1076、147125–147671；删到底那一批每盘写 10 → 12（`tr@0`、`tr`），续做那一份 12 → 14；小盘补足 1007 = 900 + 8 + 96 + 3（35 × n_记录 那一截不补，同墓碑当初；取 1008 时 240 槽那几格实测多一槽），`admission_formula_on_the_mounted_state.rs` 两格「可用 X 槽」各 +3（11 → 14、−39 → −36，标签 6 每盘一份 49193 → 98380）；384 槽小盘关准入那一格落点墙：写到 30 个单元、数据单元 19 被拒（原 26 个）。checker 档的见第八节。

## 六、验证（第 4 步那几样）

- cargo 调用 10 次：`cargo clippy` 三次（改穷举 match）、`cargo test --offline --no-run -p singlefs-harness -p singlefs-core -p singlefs-checker -p singlefs-format` 四次、末尾 `cargo clippy` 三次（前两次各抓到一处：多余 import、`identity_op`）。测试二进制不经 cargo、逐个经 `scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 4 <二进制>` 跑（7 路并行），日志在 `runs-4/`。
- `cargo fmt --all -- --check`：退 0，输出空。
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 七条：退 0，末尾原样：
  ```
      Checking singlefs-harness v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-batch50/m3/b/crates/singlefs-harness)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
  ```
- `cargo build --offline --all-targets`：没跑（预算）。
- 测试二进制 184 个：182 个退 0；`admission_raises_the_floor_before_refusing`（15 过、10 红、1 忽略）、`mount_transparency_judged_field_by_field_after_a_writable_mount`（47 过、1 红）退 101。慢档 67 条没跑，其中动过的文件里有 `rollback_by_a_forward_publish.rs` 22 条、`admission_checkpoint_cost_per_device_paths.rs` 1 条、`admission_raises_the_floor_before_refusing.rs` 1 条、`unlink_writes_per_transaction_do_not_grow_with_the_file.rs` 1 条（钉每批写数，最后一批多写退役两个单元，钉值可能要跟着变）。

## 七、没修好的 11 条（窄盘造形）

都在「不为环窗口补足、准入关着造盘面」的窄盘那一族（单元区 256 / 192 槽、覆盖写 2 次、14 个单元，第十位实现员探针挑的形状）：

- `admission_raises_the_floor_before_refusing.rs`：`a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write`、`a_continuation_step_failing_on_its_journal_record_write_hands_back_every_landed_write`、`a_continuation_step_failing_on_its_root_slot_write_hands_back_every_landed_write`、`a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_write`、`a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims`、`a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version`、`a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue`、`after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the_placement_before_acquisition_writing_nothing`、`after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor`、`an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version`
- `mount_transparency_judged_field_by_field_after_a_writable_mount.rs`：`a_mount_whose_continuation_raised_the_floor_leaves_the_effective_floor_at_the_last_one_the_continuation_raised_to`

为什么红：这几块盘不补环窗口，挂载那一判每盘扣项比之前多约 42 槽（标签 6 退役 3 槽 + ckpt_cost 退役 3 块进保留池 + 切换预留 12 × 3 = 36 槽），物理上能落的一个没多；原来「取号之前放行、续做里落得下那一判拒、推一串之后走完」变成「取号之前就不够、挂载先推一串」，后面的 txg、份数、推几串全变（例：卸载掉电那两条现行 txg 21 → 28；续做推的串 0 条 → 下标越界）。
我试过的：在副本里把单元区槽数与覆盖写次数接成环境变量（只在副本，交补丁前删干净），单元区 256–330、覆盖写 0–3 扫了：没有一格让 11 条一起过（256、261–265 只过 1–3 条）；单元数没扫（要再编一次）。
要做的：照第十位实现员那次重扫四维网格挑新形状，再改每条的份数、txg 钉值。交主 agent 另派。

## 八、受影响的层 0 流与崩溃枚举用例

（checker 的判定集合改了：I-9.4 第三句删掉、I-9.16 ④ 12 种、I-9.27 加种类 13、新立 I-9.31–I-9.35。都没跑，下面是推的。）

- 层 0 流的字节全变：新池新建文件那次树表 12 条、声明长度 2400、水位 65549；`crash-case:layer0-first-stream`、`layer0-second-stream`、`layer0-multi-record-publish-stream`、`layer0-tree-split-streams`、`layer0-position-addressed-tree-streams`、`writable-mount-of-an-io-min-8192-pool`、`instance-table-sweep-mount`、`instance-table-sweep-phases-remount`、`floor-raise-pushed-by-the-session`、`c561-sigma-full` 的输入指纹全变、要整份重跑；态数推的不变（写数不变），带准入的那几处（`floor-raise-pushed-by-the-session`）推几串可能变。
- `crash-case:layer0-unlink-after-the-multi-record-publish-stream`：最后一批删掉唯一那片叶，同一次发布多写退役树容器与根（每盘多 2 个单元，harness 那一份量到 10 → 12）；崩溃态数变多；退役树不是零根的态上池级 checker 新判 I-9.31–I-9.35（推的：合法流上全成立）；GPU 在这些态上记判不了（码 8），见设计问题第 1 条。
- 小盘几何：三档盘宽变了（补足 1007），随机历史的流与崩溃放量（`crash-case:crash-injection-fast-tier`、`read-faulted-states-never-hold-the-witness-criterion`，`crash_injection_campaign.rs`、`bad_disk_input_campaign.rs`、`crash_injection_writable_mount_after_the_crash.rs`、`crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs`、`record_checker_breaks_the_history_at_the_power_loss_seam.rs`）钉值会跟着变；256 / 240 槽两档分配记录树叶 2 → 3 片（N_0 4 → 6）。`crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs` 用第七节那一族窄盘形状，推的会红。
- GPU 对拍 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`：改了两处钉值（未登记种类 13 → 14、GPU 表容量字面量）；含 `unlink_to_the_end` 的流会有码 8 的态，那条文件头写「一个状态都不许判不了」，推的会红。
- I-9.4 第三句删掉：推的合法流上原来就没有靠它红的态。

## 九、变异表

- 删（按名字）：`实审 B2 第 9 条（I-9.4 第三句）：左容器的最大 key 小于右容器号不判`、`实审 B2 第 9 条（I-9.4 第三句）：判严一格（左最大 key + 1 < 右容器号），右容器号恰比左最大 key 大一的合法两片误红`。
- 替换 36 行（锚点随源码变了、或点名的测试改了名；改坏的意思不变）：`D8 已定项 8 头 ID…` 两行、`三·2 第 1 条…` 三行、`三·2 第 7 条…` 两行、`树 ID 整体挪开…` 三行（起点改回 11 之后十三棵 11..23、水位 24）、`第 5 批 GPU 判器：派活给每个状态的草稿区…`、`三·2 inode 树删一条…` 四行（照 M3 新写法重写锚点）、`三·2 第 6 条…` 九行、`三·2 第 5 条：冷走读的映射条目数不算谱系树`、`三·2 删除批第 12 批代码轮 U8…`、`C673 ⑤ I-9.27 判法…` 两行、`第 10 批 10b GPU 判器…` 四行、`26-3 挂载透明 续做规则的种类集合退回 {1, 2}`、`三·6 删除历史…`、`第28批 I-9.16 ④ 退回只要种类 1–4…` 两行。全文 `patch/mutations-replacements.tsv`。
- 追加 37 行，名字都以「第 50 批」开头，全文 `patch/mutations-append.tsv`。
- 别的会话的行（形式化线第 3383、3407、3496–3499 行、末行）没动；锚点在副本里各恰好命中一次（核过）。
- 副本整张表 3584 行逐行核「原文恰好命中一次、点名的测试函数在、`--test` 目标文件在」：坏 12 行，全是 W3 打完就有的那 12 行，本批没新增坏行。

## 十、停下交主 agent 的设计问题

1. GPU 只判 I-9.31、其余回落 CPU，与现行两条规矩相撞：`crash_amplification.rs` 定「配了 GPU 就一定要 GPU 判，做不到就报错，不退回 CPU」（用户 2026-09-29），GPU 对拍文件头「一个状态都不许判不了」。含删到底的流有退役树不是零根的态，崩溃放量带 GPU 那一趟会报错、对拍会红。要么 I-9.32–I-9.35 也上 GPU，要么给那两条开例外。
2. R3 验收「删空最左一片那次新写的槽数，20 片与 40 片钉同一个数」在发布路径上走不到：纯 M3 下最左那片低于 117 条就借，删不空（除非整棵只剩它）。单元层钉了；发布路径上有挪位的形状要先删一百多次，没写。要不要改验收。
3. 「贴边建满后 unlink 一次不超卖」那条没写成：harness 能填满盘的只有顺序写，一次重写整份文件、准入按整次判，贴边只贴到整份文件粒度（拒的那一刻每盘还剩约 243 槽）；真贴到一槽时 unlink 的意图那次是写入类发布、先被准入拒，第一片退役容器开不出来（判决那一形在模型里成立，是因为模型的意图不按写入类判）。另：标签 6 退役那 3 槽是常驻预付、开出第一片之后仍是 3 槽，第一片容器与根开出来实占多 2 槽却不从预付里扣，贴边时可用会不会掉到负（判决第六节「被攻过零轮」），推不出。

照最近先例自己定的（不改条款意思）：I-9.32 每片 ≤ 959 取自正推 K5.1；I-9.35 父版规则照 K5.1；一次发布多条退役记录的次序 = 那几片在上一版的叶序；退役树头 ID 配不上 inode 树时 I-9.35 报判不了（照墓碑那几条）；小盘补足取 1007 不取 1008。

## 十一、kb 要跟着改的（我没写 kb）

- `.claude/kb/decisions/08-核心索引结构.md`：已定项 6（M3：左吸收右、均分借、借完从借出那片看合并、N1、删空摘掉；文件头那句）；已定项 9（种类码 13）；已定项 11（树 ID 65548、水位 65549、十三棵连号）；已定项 8 ②（发号次序十三棵）。
- `.claude/kb/decisions/18-块里携带什么信息.md`：已定项 11（打包记录类型 3 = 退役记录，宽 34）；已定项 24（退役树形态、959 条一片、第 6 条列的零根注册的树加退役树）；两个 `format-const:` 标记（`RETIREMENT_RECORD_BYTES`、`RETIREMENT_CONTAINER_RECORDS`）。
- `.claude/kb/decisions/03-空间分配.md` 已定项 2：标签 6 式子加退役那一项。
- `.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1：W 加 6 + H_退役；已定项 4：ckpt_cost 加 2 + 1 + H_退役，映射删插数加 2 + H_退役 把。
- 不变量表：I-9.4 删第三句；I-9.16 ④ 11 → 12；I-9.27 加种类 13；新增 I-9.31–I-9.35；第一版实现条数 85 → 90。
- 新池新建文件字节表（树表 12 × 200、2400、水位 65549）与写账表（删到底那一批多写退役两个单元）。
- D23 已定项 14 不改（判决 X1 甲），可补一句「退役树同墓碑树、谱系树随现行那一版」。

## 十二、主 agent 要跑的命令

工作目录都是副本 `~/.local/share/singlefs/work/m3-batch50/m3/b`（变异表已是本批之后的那张）：

1. 构建（预算用满没跑）：`bash scripts/capped.sh 31 cargo build --offline --all-targets`，再 `bash scripts/capped.sh 31 cargo build --offline -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes`；看退出码 0。
2. 证红（名字清单每行一个，在草稿目录）：
   ```
   cd ~/.local/share/singlefs/work/m3-batch50/m3/b
   for crate in singlefs-core singlefs-format singlefs-checker singlefs-harness; do
     mapfile -t names < ../prove-red-$crate.txt
     bash scripts/prove-red.sh --copy "$PWD" --memory 8G $crate "${names[@]}"
   done
   ```
   每行判定看「抓到」，整次退 0。`26-3 挂载透明…` 那一行跑整个 `mount_transparency_judged_field_by_field_after_a_writable_mount` 二进制，基线现在红（第七节那 1 条），会被判基线红停下，等第七节修好再证。checker 档那 5 行（`--features verdict-store,gpu`）留给 `checker-tier-crates-mutation-replay`。
   新测试 → 证它的变异行：
   - `the_final_batch_that_empties_the_only_leaf_writes_one_retirement_record_and_every_retirement_invariant_holds` ← `第 50 批 死亡写侧：身份消失的那次发布不写退役记录`
   - I-9.31 / I-9.32 / I-9.33 / I-9.34 / I-9.35 漏写 / I-9.35 多写那六张坏镜像 ← `第 50 批 checker I-9.31…`、`…I-9.32…`、`…I-9.33：谱系上同一身份两条不判`、`…I-9.34…`、`…I-9.35：漏记不判`、`…I-9.35：多记不判`
   - `unlinking_rolling_back_and_unlinking_again_…` ← `第 50 批 checker I-9.33：不按谱系滤…`
   - `rolling_back_past_the_birth_of_a_leaf_…` ← `第 50 批 checker I-9.35：父版恒取 txg − 1…`
   - `a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_…` ← `第 50 批 checker I-9.27：退役树（种类 13）条目不判头 ID`、`…头 ID 配不上 inode 树时照「没有退役记录」判漏记`
   - `a_borrowed_right_container_numbered_below_the_largest_key_on_its_left_holds_every_invariant` ← `第 50 批 checker I-9.4：第三句…又判回来…`
   - `the_left_leaf_absorbs_…` ← `第 50 批 死亡写侧：被吸收的右片不写退役记录…`、`第 50 批 M3：两片之和恰好 233 不并…`
   - `a_leaf_below_the_floor_borrows_…_and_retires_nothing` ← `第 50 批 N1：借之后右片的分隔 key 不改…`、`第 50 批 M3：低于下限也不借`
   - `the_lender_absorbs_…` ← `第 50 批 M3：借完不从借出的那一片接着看合并`
   - `tree_table_holds_twelve_entries_keyed_by_tree_identifier` ← `第 50 批 R1：新池新建文件那次不零根注册退役树…`
   - core / format 单测：34 字节布局 ← `R1：退役记录死亡 txg 段与容器出生代段写反`；开第一片 ← `R1：退役记录的死亡 txg 取这次发布的前一个 txg…`；整棵照抄 ← `退役树：没有新记录的发布照样重写开放容器`；满了关片跨两片 ← `退役树：开放容器满了不关…` 与 `容器树多条追加：…只记最后那一片`；根满长高 ← `退役树：根满不长高…`；同一身份两条 ← `退役树只追加：…盖掉已有的记录`；容量开关 ← `退役树：只供测试的容量收 0 条`；容器树多条 ← `…（容器树自己那条用例）`；吸收 ← `M3：左吸收右时身份留给右片…`；借 ← `M3：借不均分（多借一条）`；借到并 ← `M3：借不均分（多借一条），一路删到并`；R3 20 / 40 片 ← `R3：删空最左一片之后右边挪了位置的片按位置重写…`；标签 6 退役 ← `R2：标签 6 不加退役那 35 字节每条记录`、`R2：标签 6 常驻不加退役树内部层数`；ckpt 退役 ← `R2：ckpt_cost 退役那一项不加退役树内部层数`；W 374 ← `R2：环窗口 W 不加退役那 6 + H_退役`；959 ← `格式：一片退役容器装的条数按 33 字节一条算`（名字前缀都是「第 50 批 」）。
   - debug 下若先撞 `debug_assert`，照实记；要断言自己红就 `--release` 再跑。
3. 第七节 11 条的重新造形：另派实现员，照 `admission_raises_the_floor_before_refusing.rs` 第 276–283 行注释那个网格重扫；`mount_transparency_…` 的 `pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices` 用同一形状。
4. 慢档（动过的文件里那几条）：`bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 31 cargo test --offline -p singlefs-harness --test rollback_by_a_forward_publish --test unlink_writes_per_transaction_do_not_grow_with_the_file --test admission_checkpoint_cost_per_device_paths --test admission_raises_the_floor_before_refusing -- --ignored`。
5. checker 档（提交时）：六道重阶段、`checker-tier-layer0-replay` 快档与全量、`checker-tier-crates-mutation-replay`，对照第八节；GPU 对拍照设计问题第 1 条的定案再看。

## 十三、没做什么

- 没走三方对抗；checker 档（`checker-tier-layer0-replay` 快档与全量、QEMU、herd7、crates 变异表、Verus 形式化验证）归提交时的主 agent；没提交；kb 一个字没写。
- 证红一条没跑；`cargo build --offline --all-targets` 与 e171 那个 bin 的 `cargo build` 没跑。
- 第七节 11 条没修好；设计问题第 2、3 条那两条用例没写。
- GPU 没判 I-9.32–I-9.35（照派发回落 CPU）。

## 十四、草稿目录里留着的

`~/.local/share/singlefs/work/m3-batch50/m3/` 下：`b/`（本批之后的副本，含 `target/`，第十二节要用它，用完由主 agent 删）、`patch/`、`patch-combined/`、`prove-red-*.txt`、`runs-*/`（测试日志）、`final/`（fmt 与 clippy 日志与退出码）、`scripts/`、`probe-*/`（窄盘探针日志）。`base/`、`w3/` 两份副本与 `patchgen.*` 临时仓交回前删掉。

## 附：两套补丁的 `--dry-run`（交回前跑的，原样）

- `python3 scripts/apply-writer-patch.py ~/.local/share/singlefs/work/m3-batch50/m3/patch-combined --root /home/fy5090/code/singlefs --dry-run`：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 3627 行`，退 0（对着主工作区此刻的那一版核的）。
- `python3 scripts/apply-writer-patch.py ~/.local/share/singlefs/work/m3-batch50/m3/patch --root <主工作区 + W3 的副本> --dry-run`：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 3589 行`，退 0。
