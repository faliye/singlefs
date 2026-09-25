# 实现员报告：impl-m2-followups2（增补 2 收口表第 58 行 `raise_rollback_floor` 同形、第 39 行那一族「取号之前的准入不算落点」、主表第 211 行变异）

时刻一律 UTC（东京 = UTC+9）。仓副本 `repo/`（08:52 从主工作区 rsync，快照 `base-crates/`）；证红在 `pv-dbg/`、`pv-rel/`、`m208*/`；最后一轮在 `final/`（10:48 从已提交成 `a1f4691` 等五个提交的主工作区重新 rsync，打补丁，追加 8 行变异）。主工作区一个字没动。续会话之后（10:48 起）核过现场：各证红日志都有 `exit=` 行；两次 `cargo test --all`（`base-test-all.log` 停在第 29 个二进制、`final-test-all.log` 停在第 12 个）随旧进程退出没跑完、各自已跑的部分无红；`final/` 上重起的那一次按主 agent 转达的用户定案在 13:08 停掉，全量留给最后统一跑（第八节）。

## 一、交付物

| 样 | 路径 | 说明 |
|---|---|---|
| 补丁 | `/tmp/claude-1000/impl-m2-followups2/impl-m2-followups2.patch` | 只含 `crates/`，8 个文件，不含 `crates/mutations.tsv`；10:48 与 13:09 两次在主工作区（HEAD `e980a21`）`git apply --check` 退出 0 |
| 追加变异行 | `/tmp/claude-1000/impl-m2-followups2/mutations-append.tsv` | 8 行整行；`final/` 上追加在主工作区 519 行之后，逐行原文恰好命中一次 |
| 第 211 行替代行 | `/tmp/claude-1000/impl-m2-followups2/row-208-replacement.tsv` | 1 行整行。派发说的「第 211 行」是实十五按 08:46 那一版表数的；今天主工作区的表里它在**第 208 行**（原文 `增补 3 第 4 件（用户 2026-09-21 定的收严）：说谎的设备许可留下的盘面不一致白名单里去掉 ["I-3.1"] 那一组`，`grep -n` 现取） |
| 报告 | `/tmp/claude-1000/impl-m2-followups2/report.md` | 本文件 |

补丁自己的 stat（主工作区上 `git apply --stat`，原样）：

```
 crates/singlefs-core/src/mount.rs                  |  313 ++++++++++++++++++--
 crates/singlefs-harness/src/history.rs             |   35 ++
 crates/singlefs-harness/src/model_comparison.rs    |   15 +
 .../src/bin/first_transaction_on_device.rs         |   93 ++++++
 crates/singlefs-harness/src/fault_injection.rs     |    4 
 ...ion_supplement_two_commit_generated_fallback.rs |   72 ++++-
 ..._transaction_supplement_three_random_history.rs |  177 +++++++++++
 ...transaction_supplement_three_fault_injection.rs |   35 ++
 8 files changed, 676 insertions(+), 68 deletions(-)
```

主工作区（HEAD `e980a21`）`git diff --stat -- crates litmus`，10:50 现跑，原样：

```
 crates/singlefs-harness/src/bin/e158_root_choice_repair.rs | 14 +++++++++-----
 1 file changed, 9 insertions(+), 5 deletions(-)
```

那是别的会话在改的实验装置，不是我的；我的改动只在补丁里、不在主工作区。开工时（08:52，提交之前）那一版是 60 个文件，原样在 `main-diff-stat.txt`。

## 二、三件做成了什么（行号是 `final/` 的，= 主工作区 + 补丁）

### 第 1 件：抬 F 那一串的失败账（收口表第 58 行「`raise_rollback_floor` 同形」）

- `mount.rs:158` `PublishSequenceFailed { cause, writes_of_persisted_publishes, writes_of_failed_publishes }`：实十五的 `PublishAfterAcquisitionFailed` 改名，可写挂载与抬 F 共用一个类型（同一个概念：自己开写入口、接连推的一串发布里有一次没做成）。
- `mount.rs:60` `MountError::RaiseFloorSequencePublishFailed(PublishSequenceFailed)`：原来的 `{ publishes_persisted, cause }` 改成同一个形态；已落盘次数 = `writes_of_persisted_publishes.len()`。
- `mount.rs:1444` `publish_sequence_failed`：失败账从写入口取的那一行（`writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),`，第 507 行变异的锚点）挪进这一处，可写挂载（`publish_failed_after_acquisition`，签名与第 508 行锚点不变）与抬 F（`mount.rs:1006`）都走它。
- 真设备二进制那一侧：**五个模式里没有一个抬 F**（`grep -rn raise_rollback_floor crates/singlefs-harness/src/bin/first_transaction_on_device.rs` 只命中补丁里新加的用例）。所以「照实十五的做法判相等」落在二进制的用例模块里：新用例 `failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count`（`first_transaction_on_device.rs:2035`）走 `second-instance` 那条路到可写挂载做完（实例 2，现行 txg 7），装注入，抬到现行的 F（0），用二进制自己的 `describe_failed_window` 判：整池第 16 次写报错 ⇒ 已落盘 13 + 失败账 2 = 15 = 设备一层，`matches=true`。二进制的 `MountError` 那一处 match 只跟着改形态。
- 另一条：`a_raise_whose_second_empty_publish_is_refused_…`（`…commit_generated_fallback.rs:391`）加三条断言——已落盘那一份就是 txg 14 那次发布自己的账、失败账为空（落点拒绝在任何写之前）、两样相加与录制器数到的写逐项相等（`sum_of_accounts` / `recorded_writes_since`，同文件 `:209`、`:220`）。

推翻条件：抬 F 那一串中途任一次写报错或被拒时，`writes_of_persisted_publishes` 与 `writes_of_failed_publishes` 之和与设备一层（或录制器）数到的写不等（今天钉的两格：13 + 2 = 15；1 份 + 0 = 录制器那几步）。

### 第 2 件：取号之前把这次挂载要写的落点算进去（收口表第 39 行那一族）

**条款推不出「算进去」怎么算，停在点名「条款没定」的错误成员上。** 读过的条款（原文整行在各文件，这里只指位置）：
- D2（RAID 条带策略） 已定项 13 的表（`02-RAID条带策略.md:224`「挂载准入」那一行）：可写挂载准入的合取之一是「实例切换的预留拿得到（那个量的式子在 D28（挂载期承诺量） 已定项 3）」；D23（journal 的角色与格式） 已定项 14（`23-journal的角色与格式.md:373`）「切换要用的块在挂载准入时预留」。
- D28（挂载期承诺量） 已定项 3（`28-挂载期承诺量.md:70` 起，`:75`「暖机」、`:76`「N_switch + 1 份…多的一份给写行那次发布的元数据」）：挂载这一串（写行 + 暖机）就在这份预留里，而暖机那一半是「R × 现算 c_max」，c_max 按已定项 4（`:90` 起，`:94` ckpt_cost 的形态）现算——树高从哪读、「记录树」指哪几棵没有条款（C363（现算保留池时树高从哪读没有条款），`checks-owed.md:316`），`crates/singlefs-core/src/admission.rs` 模块文档第 11–14 行也写着这两个数由调用方给、不替它定。
- D28（挂载期承诺量） 已定项 1（`:14`）与 D3（空间分配） 已定项 7 合取表第 1 条（`03-空间分配.md:134`）的「可用 ≥ 需求」：「需求」怎么摊到每块盘没有条款（C370（需求、可用与 df 没有共同单位），`checks-owed.md:322`）；D3（空间分配） 已定项 12（`:241`）只定登记位与两道闸的先后。

做成的样子：
- `mount.rs:143` 新成员 `MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire, publish_index, warm_up_publishes_planned, unit, refusal }`，文档注释写明是哪几条没定、第一版不算那条式子。
- `mount.rs:1268` `placements_of_the_publishes_after_acquisition_on_a_copy`：取号之前在分配器的**拷贝**上把取号之后那一串（写行一次、暖机按 `warm_up_publish_txgs` 那一份计划）逐次走一遍分配动作——释放这一次换下的落点、按 bump 次序给每个重写的角色取落点（`take_placement_for_role`，`:1241`，与发布路径同一条 `TransactionUnit::placement` 政策）、记这条根盖掉的根环槽。不碰盘、不读盘。取不到就在取号之前返回新成员（`establish_instance` 里 `:1506` 那一处，紧跟在第 105 行变异的锚点之后，锚点原文没动）。
- 判定与后面的写读同一份输入：拷贝取自取号之前那一刻的分配器，取号不碰分配器，真发起来从同一个分配器接着走；`mount.rs:1655` 在这一串发完之后断言拷贝上取的与真发取的逐次相同（`placements_taken_by`，`:1348`）。这条断言让每一次成功的挂载都在核「取号之前判的就是真发的那一串」——第 P4 条变异（拷贝上少取一个角色）就是它判出来的（第五节）。
- 胶水：`model_comparison.rs` 把新成员按分配器的原因映射（每块盘上都没有 = 单元区墙），`history.rs` 的成员名带原因；随机历史多一档盘宽 `HistoryDeviceWidth::UnitAreaOf240Slots`（`history.rs:84`）。

这条历史做成的用例（`…three_random_history.rs:498`）：两块单元区 240 槽的小盘，第一个文件之后在 mkfs 那条会话里覆盖写 22 次（内容为空），再可写挂载。这段是实十五报告里那几个 240 槽种子（首个 7463871032432355113，第 28 步「拒绝之前写了盘」）在这个副本上用 `shrink_to_reproduction` 收缩出来的最短复现（原样输出在 `logs/explore-240.log`）。今天：挂载那一步返回 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided(NoFreeSlotOnAnyDevice)`，那一步前后镜像逐字节相同、录制流一步没多、历史跑完、每一步之后池级 checker 绿；同一份盘面另起两块内存盘直接调 `mount_writable`，钉住 `instance_to_acquire = 2、publish_index = 1、warm_up_publishes_planned = 2、unit = MappingTree、refusal = NoFreeSlotOnAnyDevice`（写行那次取得到，第一次暖机的第三个固定点取不到），录制流为空、两块盘逐字节不变。改之前（第 P1 条变异就是把返回去掉）：历史以「拒绝之前写了盘」收尾。

拷贝上为什么要连释放一起走：`…three_random_history.rs:608` 那条用例。240 槽，第一个文件之后可写挂载一次、再覆盖写 21 次，回退到根环里最旧的那条根（其余 23 条全被抛弃）：写行那次的根正好盖掉回退目标那一槽，环里再没有比它旧的有效根，按可再分配谓词当场回收写行换下的几片，暖机只有它们装得下。拷贝上不释放（第 P3 条变异），这个真发得出来的回退在取号之前被拒掉。这段历史是在草稿副本上扫「回退到环里最旧的根」那几档（盘宽 240 / 256 / 384、覆盖写 20–34 次、中间挂载一次的位置，共 465 格）挑出来的，扫描结果 `logs/probe-grid-repo.txt` 与 `logs/probe-grid-mut-e.txt`。

推翻条件：
- 一段历史上可写挂载（或回退）在取号之后被落点拒绝（`MountError::Publish(PublishError::PlacementRefused …)`），而那一串里没有一份换下的单元被隔离——说明拷贝与真发分叉了。
- `mount.rs:1655` 那条断言在任何一段历史上触发。
- 拷贝上判「取不到」的挂载，真发起来发得出来（同上，没有隔离的前提下）。

## 三、停下交主 agent 的问题

1. **取号之前的准入怎么把这次挂载要写的落点算进去——条款没定，我没定。** 第一版只拒「这一串自己的落点在拷贝上真取不到」的挂载，一个真发得出来的挂载都不多拒；它**不是** D2（RAID 条带策略） 已定项 13 那条「实例切换的预留拿得到」。可选的方向（我不选）：按 D28（挂载期承诺量） 已定项 3 的式子算预留（先要 C363（现算保留池时树高从哪读没有条款） 给出 c_max、C370（需求、可用与 df 没有共同单位） 给出需求怎么摊）；照收口表第 39 行用户定的「保留上界」口径按角色 × 跨度加（会拒掉拷贝上取得到的池）；或把今天的「拷贝上真取」定成条款。今天的做法每次挂载多一次分配器拷贝（两块盘各一张位图），代价没量。
2. **拷贝上没做「释放前读盘核校验和、对不上的隔离」**（要读盘；故障注入按读的序号摆注入点，多读一遍会挪动已有用例的注入落点）。只在一格上有差别：这一串里有一份换下的单元核出对不上而被隔离，**并且**这一串自己的根把环里比它旧的有效根全盖掉（回退到环里最旧的那条根、其余全被抛弃）。那一格拷贝上判「取得到」、真发可以在取号之后被落点拒绝；`mount.rs:1655` 的断言在「这一串里有一份被隔离」时不比（`:1646` 的 `some_copy_was_quarantined`）。这是留着的缺口，没有用例（要坏盘加那段回退历史）。
3. **拷贝上看见了另一种取号之后才报的错，我没拦**：带文件的一版上写行那次经映射查换下的落点报错（`transaction::placements_to_release_via_mapping` 的六种，坏盘上才有）时，拷贝交 `ReleaseCheckFailedBeforeTheFirstPlacement`、不判不拦，发布路径取号之后照旧报同一个错、实例代号照旧烧掉。挪到取号之前是同一类改动，但不在派发里。
4. **真设备二进制没有抬 F 的模式**，第 58 行「二进制那一侧判相等」只能落在二进制的用例模块里（第二节第 1 件）。要不要给虚机档加一个抬 F 的模式（连带宿主检查、门禁 55 号的预录样本），交你定；加的话归 crash-verifier 那一侧。
5. **第 208 行替代行的取样点只钉了一个种子**，那一格为什么只剩 I-3.1、丢的是哪一份，没有逐字节追（第六节）。
6. 补丁让 `mount.rs` 多依赖两样 `transaction.rs` 的公开入口：`placements_to_release_via_mapping` 与 `TransactionUnit::placement` / `PlacementRule`。实十六在改 `transaction.rs` 的释放核验，这两处签名一变，合并时这里跟着改。`transaction.rs`、`allocator.rs`、`walk.rs`、`recovery.rs` 我一行没动。
7. kb 里该跟着改的（我不写 kb）：收口表第 58、39 行的状态；`checks-owed.md` 第 495 行 C516（抬 F 那一串发布被拒时前面几次已落盘） 那一格里写的 `MountError::RaiseFloorSequencePublishFailed { publishes_persisted, cause }` 形态（今天是 `RaiseFloorSequencePublishFailed(PublishSequenceFailed)`）。

## 四、断言与 `expect`「为什么走不到」（定义第 6 步）

- `mount.rs:1655` `assert_eq!(&placements_taken, planned, …)`：只在拷贝走完（`placements_planned` 是 `Some`）且这一串里没有一份被隔离时比。构造保证：拷贝是取号之前那一刻的分配器（`:1506`），取号（`acquire_expected_instance`）不碰分配器；两边的分配动作逐步相同——写行换下的落点两边都由 `placements_to_release_via_mapping` 对同一份内存里的上一版与同一份记录算（拷贝上走通了，真发那一遍同输入同结果），暖机换下的是上一次取到的同角色落点（真发那一遍经映射查到的就是这个进程刚写的那几片），角色与次序同一份（带文件的一版：`PublishShape::ROW_PUBLISH` / `EMPTY_PUBLISH` 的 `rewritten_roles`，与 `PublishPlan::resolve` 对「不写文件、不碰 inode 树」的计划给出的清单逐项同序；树表 0 条的一版：实例表在前、分配记录树节点在后，同 `publish_instance_table_after_the_release_check`），记根的 txg 同一串；隔离那一步只在被隔离的槽被回收之后才挡东西，条件里排掉了。调用点只有 `establish_instance`，它的调用方是 `mount_writable` 与 `mount_rollback`。实证：`final/` 上停下之前跑完的 25 个测试目标、与动到的 6 个二进制（第八节）里的每一次挂载都过了这条，其余约 40 个二进制留给统一那一次；P4 变异把拷贝上树表 0 条那一版的角色少写一个，formatted_pool 那个二进制 13 条里 8 条红在这条断言上（第五节）。
- `mount.rs:1369` `slot_shared_by_both_location_entries(…).expect(…)`：只对根记录里出生在这一次 txg 的那两条指针（这个进程这次发布刚写的实例表与分配记录树节点）调，它们的两条位置条目由 `PoolWriter::location_entries` 按一个槽写，按构造同槽；盘上读来的指针不走这里。

## 五、每条新测试「改坏哪一行 → 哪条断言红」

做法：两份副本 `pv-dbg/`、`pv-rel/`（各用自己的 target），从 `repo/` rsync；先跑不改动的一遍取基线红集，再逐条按 `prove/specs/<名>.old|.new` 改坏一处、跑那条测试所在的**整个**测试二进制（`--no-fail-fast`，不带过滤）、从 `repo/` 拷回原件并 `touch`、逐字节核相同（`prove/mutate.py`，`prove/run.sh`）。`pv-dbg` 跑 debug；随机历史、故障注入与 `--lib` 三个在 `pv-rel` 跑 `--release`（机器上别的会话的扫描进程占满了核）。被改坏的代码里没有 `debug_assert`。日志逐条在 `logs/prove/<标签>.log`。

基线（不改动）：`dbg-base-B1` first_transaction_on_device 13 过；`dbg-base-B2` commit_generated_fallback 5 过；`dbg-base-B4` step_three_formatted_pool 13 过；`rel-base-B3` random_history 21 过 2 忽略；`rel-base-B5` fault_injection 9 过 1 忽略；`rel-base-LIB` singlefs-harness --lib 67 过。基线红集为空。

| 变异（`mutations-append.tsv` 第几行） | 改坏哪一行（`final/` 行号） | 点名测试红在哪条断言（证红时副本里的行号） | 同一二进制同时红的 |
|---|---|---|---|
| R1（第 1、2 行） | `mount.rs:1008–1011` 抬 F 交已落盘账那四行换成 `Vec::new()` | B1：`failed_raise_…`（`first_transaction_on_device.rs:1752`「同一段里失败之前已经落盘的发布各一行」，只剩失败账一行、窗口 2 ≠ 设备 15、`matches=false`）；B2：`a_raise_whose_second_empty_publish_is_refused_…`（`…commit_generated_fallback.rs:432`「这一串在被拒之前已经落盘了一次（txg 14）」left 0 / right 1） | 无（B1 12 过 1 红；B2 4 过 1 红） |
| R2（第 3 行） | `mount.rs:1452` 失败账换成 `Vec::new()`（锚点同第 507 行） | `failed_raise_…`（`:1757`「失败账一份」） | `failed_writable_mount_…`（同一行） |
| P1（第 4 行） | `mount.rs:1514–1524` 拷贝上取不到那一臂不返回、照旧取号 | `a_writable_mount_whose_own_publishes_…`（`…three_random_history.rs:523` `run.ending` left `NewFinding { ModelDisagreement { aspect: "拒绝之前写了盘" } … Operation(22) … }` / right `Completed`） | 无（20 过 1 红） |
| P2（第 5 行） | `mount.rs:1326` 拷贝上不记根 | 同上那条，红在直接调 `mount_writable` 那条断言（`:573`：拷贝上被拒的换成 `unit: AccountingTree`） | `rolling_back_to_the_oldest_ring_root_…`（`:633` left `Refused { … PlacementRefusedBeforeAcquisitionMountAdmissionUndecided(NoFreeSlotOnAnyDevice) }`）、`unit_area_wall_sampling_…`（`:391`，256 槽 32 段里新发现 31） |
| P3（第 6 行） | `mount.rs:1311` 拷贝上不释放换下的落点 | `rolling_back_to_the_oldest_ring_root_…`（`:633` left `Refused { … PlacementRefusedBeforeAcquisitionMountAdmissionUndecided(NoFreeSlotOnAnyDevice) }`，right `Applied(Mounted { instance 3, publishes 2, … rewritten_from_released: 8 … })`） | 无（20 过 1 红） |
| P5（第 7 行） | `model_comparison.rs:268–270` 新成员映射成模型没有的理由 | `a_writable_mount_whose_own_publishes_…`（`:523` left `NewFinding { ModelDisagreement { aspect: "模型说该成、实现拒了" } … }`） | 无（20 过 1 红） |
| P4（第 8 行） | `mount.rs:1293–1296` 拷贝上树表 0 条那一版写行只取实例表 | `a_formatted_pool_mounted_twice_…`（`mount.rs:1655` 那条断言；这条变异把四行换成一行，panic 报在改坏之后的 `:1652`：left `[[(InstanceTable, 50240), (AllocationTree, 50242)], []]` / right `[[(InstanceTable, 50240)], []]`） | formatted_pool 另外 7 条同一处红：`a_third_writable_mount_…`、`the_third_writable_mount_keeps_…`、`rolling_back_to_a_warm_up_root_…`、`root_record_of_a_file_version_…`、`writable_mount_after_a_crash_right_after_acquiring_…`、`rolling_back_before_any_file_version_…`、`writable_mount_after_a_crash_between_warm_up_…`（5 过 8 红） |
| M208（替代行） | `fault_injection.rs:1913` 白名单去掉 `["I-3.1"]` | `a_swallowed_write_after_which_the_checker_flags_only_i_3_1_…`（`…three_fault_injection.rs:282`，今天 `:283`「说谎的设备留下的 I-3.1 要被白名单豁免，不算新发现」） | 无（8 过 1 红 1 忽略）；`--lib` 里 `only_the_two_whitelisted_projections_of_a_lost_write_are_excused`（`fault_injection.rs:2807`，今天 `:2809`「白名单里这一组该豁免：CheckerViolations { invariants: ["I-3.1"] }」，66 过 1 红） |

第二节第 1 件里 `a_raise_whose_second_empty_publish_is_refused_…` 新加的三条断言（账等于 txg 14 自己的账、失败账为空、与录制器相等）没有单独的变异；R1 下它红在前面那条「已落盘一次」上。

## 六、主表第 211 行（今天第 208 行）变异：分类与处置

**分类：取样点不敏感（`.claude/rules/mutation-sampling.md`「三类，判据不同」表第三行）。** 依据，全在不打补丁的主工作区副本上量（`m208/`、`m208-mut/`，08:52 那一版，第 208 行涉及的文件与今天的提交逐字节相同）：

- 不是锚点走不到：白名单那一判对每一次「说谎的设备留下盘面不一致」都判。大档（不改动，`--release`，这个测试周期的种子基起 512 段、每段 30 步、注入 6 次，174.1 秒，`logs/m208-large-tier-unmutated.log`）原样：`说谎的设备丢掉一份内容之后盘面不一致：550 次（设备丢的，不算新发现）`、`CheckerViolations { invariants: ["I-2.1", "I-4.8", "I-7.4"] }：524 次`、`CheckerViolations { invariants: ["I-3.1"] }：26 次`。
- 不是等价变异：`["I-3.1"]` 那一组在真抽出来的注入上出现（上面 26 次；在一份只加了 `eprintln` 的副本上逐个记下是 13 个注入点，`logs/m208/probe-i31-seeds.txt`，全是 `WriteIsSwallowed`；26 = 13 × 2，逐一核过的那一段是每个注入点记两次——带注入跑的那一遍每步之后的 checker 与注入之后的镜像上那一遍各一次）。去掉这一组，这 13 处都会变成新发现。
- 点名的测试在它自己的取样点上看不见：快档（24 段 × 20 步 × 4 次注入）改坏之后照样绿（`m208-mut/` debug，`logs/m208/mut-fault-injection-binary.log`：`test result: ok. 8 passed; 0 failed; 1 ignored`），它的报告里说谎那一格原样 `说谎的设备丢掉一份内容之后盘面不一致：16 次`、只有 `CheckerViolations { invariants: ["I-2.1", "I-4.8", "I-7.4"] }：16 次`。
- 另有一条会红的：`--lib` 里 `only_the_two_whitelisted_projections_of_a_lost_write_are_excused` 直接拿 `["I-3.1"]` 问白名单，改坏之后红（`logs/m208/mut-harness-lib.log`：`test result: FAILED. 66 passed; 1 failed`）。它只核「名单里有这一组」，不核「说谎的设备真会留下这一组、而且被豁免」。

**处置：补一个敏感的取样点。** 新用例 `a_swallowed_write_after_which_the_checker_flags_only_i_3_1_is_excused_as_what_a_lying_device_may_leave`（`…three_fault_injection.rs:271`）：大档那 13 处里取种子 7463871032432355306 那一段（同一组规模，被吞掉的写是整池第 107 次写调用、落在 `step_index` 6 那一步覆盖写上），跑一段，断言没有新发现、说谎那一格里 `["I-3.1"]` 记 2 次。改坏之后它红（第五节 M208）。`fault_injection.rs` 白名单上那段文档注释原来写「白名单少一组…快档判红」，照实改成快档里那一组一次都没有、取样点是这条用例（补丁里 `fault_injection.rs` 那 4 行就是这个）。这一格为什么只剩 I-3.1、被吞掉的是哪一份，没有逐字节追。

**第 208 行的整行替代**（`row-208-replacement.tsv`，六列以 Tab 分隔；第 3、4 列与原行逐字相同，只改第 1、5、6 列）：

```
增补 3 第 4 件（用户 2026-09-21 定的收严）：说谎的设备许可留下的盘面不一致白名单里去掉 ["I-3.1"] 那一组；那一组的取样点（一段被吞掉的写之后只判红 I-3.1 的历史）判出	crates/singlefs-harness/src/fault_injection.rs	    &[&["I-2.1", "I-4.8", "I-7.4"], &["I-3.1"]];	    &[&["I-2.1", "I-4.8", "I-7.4"]];	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_swallowed_write_after_which_the_checker_flags_only_i_3_1	a_swallowed_write_after_which_the_checker_flags_only_i_3_1_is_excused_as_what_a_lying_device_may_leave
```

顺带看到、没追的：同一次大档（不改动）以 `以「已知红」收尾 {0: 179}、新发现 47` 收尾、判红（`test result: FAILED`）；我读到的几条新发现全是 `write_is_swallowed` 之后白名单之外的签名（例如 `["I-2.1", "I-3.10", "I-4.8", "I-7.4"]`、`["I-2.1", "I-4.8", "I-7.4", "I-9.1"]`、模型对不上「抬 F 的上限」「冷启动读回」「写的实例表行」）。大档不在门禁里、也不在派发里，原样日志在 `logs/m208-large-tier-unmutated.log`，交你定归谁。

## 七、`crates/mutations.tsv` 要主 agent 处置的已有行（行号是主工作区 `e980a21` 那一版的，519 行）

只追加、不改别人的行，所以列在这里。`final/`（主工作区 + 补丁 + 追加 8 行，527 行）加上替代行逐行数原文（`scratch/check-anchors.py`，原样末两行）：

```
mutations.tsv:471: 命中 0 次：C516（抬 F 那一串发布被拒时前面几次已落盘）：被拒时报出的已落盘次数写死成 0
核了 523 行，不是恰好一次的 1 行
```

| 行 | 为什么 | 处置 |
|---|---|---|
| 208「增补 3 第 4 件（用户 2026-09-21 定的收严）：说谎的设备许可留下的盘面不一致白名单里去掉 ["I-3.1"] 那一组」 | 点名的快档取样点不敏感（第六节） | 换成 `row-208-replacement.tsv` 那一行 |
| 471「C516（抬 F 那一串发布被拒时前面几次已落盘）：被拒时报出的已落盘次数写死成 0」 | 抬 F 的错改成随账交出，原文（`publishes_persisted,` 那一段）命中 0 次 | 删；追加的第 2 行接替（同一个用例，红） |

实十五那几行（今天第 507–511 行）的原文照旧各命中一次：第 507 行那一行搬进了 `publish_sequence_failed`（`mount.rs:1452`），它的变异现在同时打掉可写挂载与抬 F 两条路，点名的 `failed_writable_mount_…` 照红（第五节 R2 那一格）。第 105 行（取号之前那道准入整段拿掉）的原文没动。

追加的 8 行（`mutations-append.tsv`，第一列原样）：

1. 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交已落盘那几次空发布的账；真设备二进制那一侧的判法（设备一层逐项相等）判出
2. 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串第二次空发布被落点拒绝时报出的已落盘份数成了 0（接替原第 471 行：已落盘次数改成随账交出，原文命中 0 次）
3. 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交写入口的失败账（与可写挂载共用那一处）
4. 增补 2 收口表第 39 行那一族：取号之前在分配器的拷贝上取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）
5. 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时不记根（不转环、不回收），拷贝上被拒的那一次与真发的不同
6. 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时不释放换下的落点（回退到环里最旧的根时写行当场回收的几片拷贝上看不见，真发得出来的回退被拒）
7. 增补 2 收口表第 39 行那一族：胶水把「取号之前在拷贝上取不到落点」映射成模型没有的理由（单元区墙那一格对不上）
8. 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点（拷贝上取的与真发的不同，挂载自己的断言判出）

第 4–7 行与替代行是在 `--release` 下证的红，第 1–3、8 行在 debug 下；被改的代码里没有 `debug_assert`，门禁 59 号按 debug 跑时判的是同一处。没有照门禁 59 号的跑法（带过滤）逐行复跑，整表归 crash-verifier。

## 八、`check.sh` 与测试二进制

`check.sh` 在 `final/`（10:48 的主工作区 `e980a21` + 补丁 + 追加 8 行）上跑，10:48:49 起，`nice -n 19 bash .claude/scripts/check.sh` 末尾原样：

```
error: could not compile `singlefs-harness` (bin "e158_root_choice_repair") due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 3 previous errors
  ✗ clippy 有告警（按 -D warnings 视为错误），或者踩了编码纪律的某一条
     → 怎么办： 上面每条告警都指着文件和行号，逐条改。编码纪律那几条的写法见 rules/code-discipline.md。
                确有必要保留的，在那一处写 #[allow(<lint>, reason = "为什么")]，理由写进 reason——
                不要整仓关掉 -D warnings（rules/command-safety.md：警告是最便宜的信号）。
exit=1
```

- `cargo fmt --check` 那一段过（`✓ 格式通过`）。clippy 报的错全在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（`-->` 指到的 3 处都在这个文件），补丁动过的 8 个文件一处都没有。e158 是别的会话在改的实验装置（10:50 主工作区 `git diff --stat -- crates litmus` 只有它一个文件），照实写。`check.sh` 停在 clippy，build / test 两段没走到。
- `cargo build --all-targets`：在 `final/` 上 10:27 那一版（主工作区提交之前 + 补丁）退出 0（`logs/final-build.log`）。
- **全量测试留给最后统一跑**（主 agent 13:07 之后转达用户的定：全部代码落定之后统一跑一次）。`final/` 上 10:48:57 起的 `cargo test --all --no-fail-fast` 在 13:08 按主 agent 的指示停掉（`proc.py stop` 停了 cargo pid 2051691 与正在跑的测试二进制 pid 3875651 `second_transaction_step_five_reuse`；外层 bash 随之退出，日志末行 `exit=143`）。停之前跑完的 25 个测试目标无失败，逐个（`logs/final2-test-all.log`）：`singlefs-checker` lib 3 过；`singlefs-core` lib 94 过；`singlefs-format` lib 5 过；`singlefs-harness` lib 67 过；bin `e156_allocation_basis_counts` 6 过；bin `e158_root_choice_repair` 28 过；bin `first_transaction_device_log_check` 4 过；bin `first_transaction_on_device` 13 过；bin `first_transaction_region_bytes` 0 个；`checker_known_bad_images` 27 过；`first_transaction_region_bytes` 4 过；`first_transaction_step_five_publish` 8 过；`first_transaction_step_one_mkfs` 9 过；`first_transaction_step_seven_layer0` 5 过 1 忽略；`first_transaction_step_six_recovery` 6 过；`first_transaction_step_two_data_unit` 3 过；`instance_acquisition` 4 过；`parallel_line_one_sequential_write` 3 过；`publish_order_matches_litmus` 1 过；`second_transaction_mapping_node_admission` 4 过；`second_transaction_parallel_line_one_layer0` 1 过 1 忽略；`second_transaction_parallel_line_one_multi_unit_file` 7 过；`second_transaction_parallel_line_one_sequential_write` 3 过；`second_transaction_parallel_line_three_many_inodes` 4 过；`second_transaction_parallel_line_two_mounted_read` 11 过。
- 动到的测试二进制在打着补丁的副本上都跑完、全绿（第五节的基线那一遍）：`first_transaction_on_device` 13 过、`second_transaction_supplement_two_commit_generated_fallback` 5 过、`second_transaction_step_three_formatted_pool` 13 过（debug，`pv-dbg/`）；`second_transaction_supplement_three_random_history` 21 过 2 忽略、`second_transaction_supplement_three_fault_injection` 9 过 1 忽略、`singlefs-harness --lib` 67 过（`--release`，`pv-rel/`）。这两份副本与 `final/` 在补丁动过的文件上只差两处文档注释（`src/fault_injection.rs` 白名单上那段、故障注入用例文件里新用例上那段，都是证红之后改的，代码逐字节相同）；`pv-dbg` 同步时故障注入那条新用例还没写，它跑的三个二进制不含那个文件。
- 补丁每一次挂载都多走一遍拷贝与一条断言（第二节第 2 件），挂载的二进制差不多就是全部；没跑到的那一半（`second_transaction_step_five_reuse` 起约 40 个）留给统一那一次。
- 登记给我的门禁阶段按派发没跑。

## 九、写过的文件

补丁里 8 个（全在 `crates/`）：`singlefs-core/src/mount.rs`；`singlefs-harness/src/{history,model_comparison,fault_injection}.rs`（`fault_injection.rs` 只改文档注释）、`singlefs-harness/src/bin/first_transaction_on_device.rs`；`singlefs-harness/tests/` 下 `second_transaction_supplement_two_commit_generated_fallback.rs`、`second_transaction_supplement_three_random_history.rs`、`second_transaction_supplement_three_fault_injection.rs`。没有新文件、没碰 `Cargo.toml`、`litmus/`、`crates/mutations.tsv`（追加的 8 行在 `mutations-append.tsv`，变异名见第七节；替代行在 `row-208-replacement.tsv`）。没碰 `transaction.rs`、`allocator.rs`（实十六）、`walk.rs`、`recovery.rs`（实十四），也没碰 e156 / e158。草稿副本里用过的探针用例（`zz_explore_tmp.rs`、`zz_probe_rollback.rs`，只在 `repo/`、`mut-e/` 里）已从 `repo/` 删掉，不在补丁里。

## 十、负载与经过里要记的

- 每次开跑前 `ps` 看过，没有 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`。别的会话的进程一直占着核：`/tmp/claude-1000/m2-keyspace-opus/bin-scan3 scan_candidates`（09:45 时 1167% CPU、10:48 时 1276%）、`bin-final`、两个 `ray::RayWorkerProc`、别的会话的 `cargo test`（`impl-m2-writepath`、`impl-m2-checker3` 副本里的测试二进制）与 gate 阶段；09:45 负载 32（32 核）。各副本用各自的 target，没等锁。
- 旧会话进程在 10:43（两份日志最后一次写入的时刻）之后退出，两次 `cargo test --all` 没跑完（`base-test-all.log` 第 29 个二进制、`final-test-all.log` 第 12 个，已跑的部分无红），各证红日志都完整（有 `exit=`）。续上之后只补了 `final/` 上那一次：10:48 从已提交的主工作区重新 rsync、打补丁、追加 8 行，`check.sh` 与 `cargo test --all` 在它上面跑（第八节）。
- 探针：`shrink_to_reproduction` 收缩 240 槽那个种子（`logs/explore-240.log`）；回退到环里最旧的根那 465 格的扫描（`logs/probe-grid-*.txt`）；大档里 `["I-3.1"]` 那几处注入点的记录（`logs/m208/probe-i31-seeds.txt`，副本里临时加了一行 `eprintln`）。

## 十一、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 crash-verifier；没提交。没新加层 0 流或崩溃点重放用例。
- 取号之前的准入按哪条式子算落点没定（第三节第 1 条）；隔离那一步没进拷贝（第 2 条）；释放判定报错那一格没挪到取号之前（第 3 条）；二进制没加抬 F 的模式（第 4 条）。
- 第 208 行那一格为什么只剩 I-3.1 没追；大档（不改动）那 47 条新发现没看（第六节末）。
- 门禁 59 号的跑法（带过滤、按表逐行）没复跑追加的 8 行与替代行，只按定义跑了整个二进制。登记给我的门禁阶段按派发没跑。
- kb、`research/` 一个字没写：收口表第 58、39 行，C516（抬 F 那一串发布被拒时前面几次已落盘） 那一格里写的旧形态，由主 agent 或 kb-scribe 改。
- 草稿没入库（实现员的写范围不含 `research/results/`）：`/tmp/claude-1000/impl-m2-followups2/logs/` 下的证红日志、探针输出、大档日志、`check.sh` 与 `cargo test --all` 的日志。要不要留由主 agent 定。
