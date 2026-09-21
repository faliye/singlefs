# 附录二：archive-rename-r1 这次改动的 diff（`git diff --cached -- crates/ research/e7-index-bench/src/bin/ research/mutations/ .claude/kb/layout/ research/scripts/` 原样输出；2026-09-21 生成）

基准：`HEAD`（提交 `3cff909b082e973cd6d2aa1cb5d7b1bf636b5f2a`）与当前暂存区（`--cached`）比较。
生成时刻：2026-09-21（UTC）。

命令：

```
git diff --cached -- crates/ research/e7-index-bench/src/bin/ research/mutations/ .claude/kb/layout/ research/scripts/
```

共 43 个文件，diff 原始输出共 1785 行。太长，按文件切成 43 段，每段标文件名与它在原始 diff 里的行区间（1 起）；内容逐字节原样、不删不改。全部 43 个文件的 `git diff --cached --name-status` 都是 `M`（修改），没有新增文件，所以没有「再附新文件全文」那一部分。

## 一、diff（按文件切段）

### `.claude/kb/layout/01-first-txn.md`（原始 diff 第 1-27 行）

```diff
diff --git a/.claude/kb/layout/01-first-txn.md b/.claude/kb/layout/01-first-txn.md
index edaaad1..c2160d9 100644
--- a/.claude/kb/layout/01-first-txn.md
+++ b/.claude/kb/layout/01-first-txn.md
@@ -387,17 +387,17 @@ D8（核心索引结构） 已定项 6（2026-09-05）定了整段：inode 树
 
 | 根槽写路径 | 录制流的段序列（每段的写数与种类；「种类」那串是产物 `kinds=` 字段的原样，每段一个步骤种类多重集，段之间用 `\|` 隔开，门禁 52 号逐字比对） | 出处 | 层 0 枚举 |
 |---|---|---|---|
-| mkfs 种根 | [m1 实例表单元 × 2 盘 + m2 树表单元 × 2 盘 = 4 单元写] 屏障 [m3 根 FUA 区域 0] [m3 根 FUA 区域 1] [m3 根 FUA 区域 2] [m4 系统配置槽 × 2 盘 × 2 槽 = 4，都是世代号 1] 屏障 ⇒ `4+1+1+1+4`，13 次操作、34 个崩溃状态，种类 `[unit_write×4,barrier]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[superblock_slot×4,barrier]` | 一；E142（第一个事务的干跑） `fn mkfs`、产物 `name=segments path=mkfs` | **不在**（G19：装置从 mkfs 之后的池起枚举；段序列由单测钉住） |
-| 第一次可写挂载取号（实例代号 0 → 1，不是根槽写路径） | [a1 系统配置槽 × 2 盘，世代号 2] ⇒ `2`，2 次操作、4 个崩溃状态，种类 `[superblock_slot×2]`；取号两写之后一道屏障（D23（journal 的角色与格式） 已定项 16 要取号两写之后、本实例第一个非系统配置写之前一道完成了的屏障；2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：`crates/singlefs-core/src/transaction.rs` 的 `acquire_instance` 写完两份系统配置自己发一道（`CommitStep::Barrier`），首次挂载上它与暖机第一次空发布开头那道背靠背、登记的段序列不变；第二次以后的挂载靠它把取号与写行那次发布的单元写隔开（2026-09-17 按代码改写）；世代号按 D22（单元原子性怎么合成） 已定项 16 逐盘 + 1，首次挂载写出的是 2 | 一（a1）；D23（journal 的角色与格式） 已定项 16；产物 `name=segments path=instance_acquisition` | 在（第六次跑起，整条流开头那一段） |
-| 空发布（暖机，第一版 2 次） | 屏障 [w1 空记录 × 2 盘] 屏障 [w2 根 FUA] [w3 系统配置槽 × 2 盘]（第二次同型，w4–w6）⇒ 两次合起来 `2+1+2+2+1+2`，种类 `[journal_record×2,barrier×2]\|[root_record_fua]\|[superblock_slot×2,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[superblock_slot×2]` | 零；D16（发布语义） 已定项 8；产物 `name=segments path=warm_up` | 在（第四次跑起） |
-| 普通发布（第一个事务） | [t1–t8 单元 × 2 盘 = 16] 屏障 [t9 记录 × 2 盘] 屏障 [t10 根 FUA] [t11 系统配置槽 × 2 盘] ⇒ `16+2+1+2`，种类 `[unit_write×16,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[superblock_slot×2]` | 零；D16（发布语义） 已定项 7；产物 `name=segments path=transaction` | 在（E142（第一个事务的干跑） 主臂） |
+| mkfs 种根 | [m1 实例表单元 × 2 盘 + m2 树表单元 × 2 盘 = 4 单元写] 屏障 [m3 根 FUA 区域 0] [m3 根 FUA 区域 1] [m3 根 FUA 区域 2] [m4 系统配置槽 × 2 盘 × 2 槽 = 4，都是世代号 1] 屏障 ⇒ `4+1+1+1+4`，13 次操作、34 个崩溃状态，种类 `[unit_write×4,barrier]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[system_configuration_slot×4,barrier]` | 一；E142（第一个事务的干跑） `fn mkfs`、产物 `name=segments path=mkfs` | **不在**（G19：装置从 mkfs 之后的池起枚举；段序列由单测钉住） |
+| 第一次可写挂载取号（实例代号 0 → 1，不是根槽写路径） | [a1 系统配置槽 × 2 盘，世代号 2] ⇒ `2`，2 次操作、4 个崩溃状态，种类 `[system_configuration_slot×2]`；取号两写之后一道屏障（D23（journal 的角色与格式） 已定项 16 要取号两写之后、本实例第一个非系统配置写之前一道完成了的屏障；2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：`crates/singlefs-core/src/transaction.rs` 的 `acquire_instance` 写完两份系统配置自己发一道（`CommitStep::Barrier`），首次挂载上它与暖机第一次空发布开头那道背靠背、登记的段序列不变；第二次以后的挂载靠它把取号与写行那次发布的单元写隔开（2026-09-17 按代码改写）；世代号按 D22（单元原子性怎么合成） 已定项 16 逐盘 + 1，首次挂载写出的是 2 | 一（a1）；D23（journal 的角色与格式） 已定项 16；产物 `name=segments path=instance_acquisition` | 在（第六次跑起，整条流开头那一段） |
+| 空发布（暖机，第一版 2 次） | 屏障 [w1 空记录 × 2 盘] 屏障 [w2 根 FUA] [w3 系统配置槽 × 2 盘]（第二次同型，w4–w6）⇒ 两次合起来 `2+1+2+2+1+2`，种类 `[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[system_configuration_slot×2]` | 零；D16（发布语义） 已定项 8；产物 `name=segments path=warm_up` | 在（第四次跑起） |
+| 普通发布（第一个事务） | [t1–t8 单元 × 2 盘 = 16] 屏障 [t9 记录 × 2 盘] 屏障 [t10 根 FUA] [t11 系统配置槽 × 2 盘] ⇒ `16+2+1+2`，种类 `[unit_write×16,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[system_configuration_slot×2]` | 零；D16（发布语义） 已定项 7；产物 `name=segments path=transaction` | 在（E142（第一个事务的干跑） 主臂） |
 | 实例切换 / 管理员回退 | 实例切换那一半 2026-09-16 已写成字节，就是「第一次之后的可写挂载（写行）」那一行（切换 = 挂载内做一次恢复再写行，D23（journal 的角色与格式） 已定项 14）；管理员回退那一半 2026-09-17 也写成字节（**装置钉住**，里程碑「第二个事务」步 4，孤立形状从第二条流推得）：[取号系统配置槽 × 2 盘，世代号 4] 屏障 [实例表单元（回退行 + 中间实例行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10] 屏障 [记录 × 2 盘，jsn 接在 R_old 那条之后] 屏障 [根 FUA] [系统配置槽 × 2 盘] ⇒ 孤立看 `2+10+2+1+2`，与写行那一行同型、多的是内容（回退行带回退位、中间实例行、影子账只住内存不写字节），**之后接新实例的暖机**（这条脚本上一次：txg 9 落盘 0、txg 10 落盘 1）；取号之后那道屏障是 D23（journal 的角色与格式） 已定项 16（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款））（D16（发布语义） 已定项 8：新实例的根覆盖两块盘之前连推空发布，与第一次挂载同型）——与普通发布 + 空发布同型，多的是内容不是步骤 | D23（journal 的角色与格式） 已定项 14；D16（发布语义） 已定项 8 | 不在 |
 | 第一次之后的可写挂载（写行） | **装置钉住**（里程碑「第二个事务」步 3 2026-09-16 落地，孤立形状从第二条流推得）：[取号系统配置槽 × 2 盘，世代号 3] 屏障 [实例表单元（写行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] ⇒ 孤立看 `2+10+2+1+2`；整条流里取号的 2 个写与上一次发布的 2 个系统配置槽写合成 4 写一段、末尾的 2 个系统配置槽写与暖机第一次的 8 个单元写合成 10 写一段（第二条流第 13–17 段 `4+10+2+1+10`）；取号之后那道屏障由取号自己发（D23（journal 的角色与格式） 已定项 16：写行那次发布有单元写，等不到空发布开头那道）；每次可写挂载都写行（实例 0 不写，第一次可写挂载要写的区间是空的），写行那次是新实例的第一次发布，之前不推抬 F 的空发布，元数据走切换预留（D18（块里携带什么信息） 已定项 11、D28（挂载期承诺量） 已定项 3；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方） | D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 16；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
 | 空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元） | **装置钉住**（步 3 落地，孤立形状从第二条流推得）：[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] ⇒ 孤立看 `8+2+1+2`，与首次挂载的暖机不同型：记账树已经存在，空发布也重写记账行连带四个固定点单元（D16（发布语义） 已定项 9），这条脚本上 c_max = 4；整条流里 8 个单元写与上一次发布的 2 个系统配置槽写合成 10 写一段；实例 2 从 txg 5 起推 2 次（txg 6 落盘 0、txg 7 落盘 1），次数按「本实例的根覆盖全部区域盘」现算、上限 3——D16（发布语义） 已定项 8 只给第一次可写挂载定了常量 2，后续挂载的次数没有条款，实做的取法等用户定 | D16（发布语义） 已定项 8 / 已定项 9；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
 | 抬 F 的空发布（D16（发布语义） 已定项 1：准入不够时先推空发布抬回退下界） | **装置钉住**（里程碑「第二个事务」步 5 2026-09-17 落地，孤立形状从第二条流推得）：与后续挂载的暖机空发布同型——[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] ⇒ 孤立看 `8+2+1+2`，根记录的 F 写成目标值、记账行是回收过释放代 ≤ F 之后的账；推到每块盘上都有一条带新 F 的根为止（这条脚本上两次：txg 15 落盘 0、txg 16 落盘 1）；第一版只有测试的强制入口，准入不够的正常触发没做 | D16（发布语义） 已定项 1；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
 | 只做过 mkfs 的池的可写挂载（取号、零单元写行、零单元暖机）再发第一个文件版本 | **装置钉住**（里程碑「第二个事务」步 3 2026-09-17 落地，2026-09-17 用户定案允许只做过 mkfs 的池可写挂载）：重开之后取号 [系统配置槽 × 2 盘] 屏障，树表 0 条 ⇒ 写行那次发布与暖机都写零个单元，与 mkfs 同一个进程里的取号、两次暖机同型，之后第一个文件版本同第一个事务 ⇒ 整条流 `2+2+1+2+2+1+18+2+1+2`，与第一个事务那条流逐段相同、闭式 262165；树表 0 条而要写实例表行、回退到树表 0 条的根都在写之前拒绝 | D16（发布语义） 已定项 9（树表 0 条时空发布写零个单元）；用例 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs` | 与第一个事务那条流的基线镜像、写表与段序列逐项相同（快用例钉住），不另枚举；崩溃状态上重开可写挂载不在层 0 里 |
 
-⚠️ 发布与下一次发布之间没有屏障：上一次发布的系统配置槽写与下一次发布的单元写落在同一段（第一个事务的第六次跑里暖机第二次的 2 个系统配置写与 16 个单元写合成 18 个写的一段，取号的 2 个系统配置写与暖机第一次开头的屏障合成开头一段，整条流 `2+2+1+2+2+1+18+2+1+2`、262165 个状态，种类 `[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]`）；mkfs 末尾有屏障、空发布开头有屏障，那两道接缝是切开的。段序列按整条录制流切，不按路径切，所以登记表按路径给的段序列是「孤立看」的形状，整条流那一行才是层 0 枚举吃的。
+⚠️ 发布与下一次发布之间没有屏障：上一次发布的系统配置槽写与下一次发布的单元写落在同一段（第一个事务的第六次跑里暖机第二次的 2 个系统配置写与 16 个单元写合成 18 个写的一段，取号的 2 个系统配置写与暖机第一次开头的屏障合成开头一段，整条流 `2+2+1+2+2+1+18+2+1+2`、262165 个状态，种类 `[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]`）；mkfs 末尾有屏障、空发布开头有屏障，那两道接缝是切开的。段序列按整条录制流切，不按路径切，所以登记表按路径给的段序列是「孤立看」的形状，整条流那一行才是层 0 枚举吃的。
 
 ⚠️ 第二条流（里程碑「第二个事务」步 0 的固定脚本，2026-09-17 做到发布 E）：取号 → 暖机 × 2 → A → B → 进程退出、重开取号 → 写行 → 暖机 × 2 → C → 进程退出、重开回退到 A 的根、取号 → D（回退行）→ 暖机 × 1 → 覆盖写 × 4 → 抬 F 的空发布 × 2 → E，第二条流的段序列 `2+2+1+2+2+1+18+2+1+18+2+1+4+10+2+1+10+2+1+10+2+1+18+2+1+4+10+2+1+10+2+1+18+2+1+18+2+1+18+2+1+18+2+1+10+2+1+10+2+1+18+2+1+2`、279 次写、2104413 个状态，没有干跑产物，装置钉住：`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` 把这个数组与闭式钉死，门禁 54 号在 release 下全量跑它、52 号核这句与用例里的数组、闭式、写数相符；到 C（26 段、789555）与到 D（33 段、791624）两个前缀各由同一份用例里只跑准备的一条钉住。前 12 段与第一条流相同，第 13 段是 B 的 2 个系统配置槽写与重开取号的 2 个系统配置槽写合成的 4 写一段（进程退出与重开之间没有屏障，录制流按设备连着记）；第 26 段同型（C 的 2 个系统配置槽写与回退取号的 2 个）。
 
```

### `crates/mutations.tsv`（原始 diff 第 28-54 行）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 1a81cdb..4c031ed 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -34,12 +34,12 @@ Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs
 步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));	            let _ = (record.device, record.slot, record.span_slots);	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
 步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg && false)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_onto_an_abandoned	rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused
 步 4：前缀第五条不判（回退行的 W 不封顶）	crates/singlefs-core/src/recovery.rs	            if high_water == 0 || record.transaction > high_water {	            if false && (high_water == 0 || record.transaction > high_water) {	-p singlefs-harness --test second_transaction_step_four_rollback -- the_rollback_row_caps	the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water
-步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	            *index == newest_index || (!abandoned && !below_floor)	            *index == newest_index || !below_floor	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
+步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	            let walked = *index == newest_index || (!abandoned && !below_floor);	            let walked = *index == newest_index || !below_floor;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
 步 5：回收不看释放代（复用窗口置 0）	crates/singlefs-core/src/allocator.rs	            .filter(|record| record.is_released && record.generation <= floor)	            .filter(|record| record.is_released)	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 5：抬 F 的上限不看第 4 新的非空根	crates/singlefs-core/src/mount.rs	    Some(newest_on_every_device.min(fourth_newest))	    Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_above	raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused
 步 5：抬 F 的空发布只推一次（F 只落在一块盘上）	crates/singlefs-core/src/mount.rs	        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");\n            existing.generation = generation;\n            existing.is_released = false;	            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),\n                generation,\n                is_released: false,\n            });	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
-步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	            *index == newest_index || (!abandoned && !below_floor)	            *index == newest_index || !abandoned	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
+步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	            let walked = *index == newest_index || (!abandoned && !below_floor);	            let walked = *index == newest_index || !abandoned;	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：F_生效 取各盘 F 最大值的最大值而不是最小值	crates/singlefs-core/src/recovery.rs	    highest_per_device\n        .values()\n        .copied()\n        .min()\n        .unwrap_or(CheckpointTxg(0))	    highest_per_device\n        .values()\n        .copied()\n        .max()\n        .unwrap_or(CheckpointTxg(0))	-p singlefs-harness --test second_transaction_step_five_reuse -- one_device_carrying_the_floor_alone	one_device_carrying_the_floor_alone_does_not_take_effect_on_remount
 步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-core/src/recovery.rs	        } else if record.checkpoint_txg != chain_start_txg_without_anchor {	        } else if false && record.checkpoint_txg != chain_start_txg_without_anchor {	-p singlefs-harness --test second_transaction_step_three_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg
 步 4：普通重开不隔离被抛弃根引用的槽（影子账只在回退那一次算）	crates/singlefs-core/src/mount.rs	        &|_| false,\n        ShadowLedger::On,\n    );	        &|_| false,\n        ShadowLedger::Off,\n    );	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
@@ -177,6 +177,7 @@ Z1-a：实例表准入按每次挂载只写一行算	crates/singlefs-core/src/mo
 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("mount_rollback"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_a_root_whose_warm_up	rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds
 增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：候选排除「低于 F」报成「树表 0 条、第一版不支持」；回退到 F 之下的写死用例判出（门禁五段各只红 1 段）	crates/singlefs-core/src/mount.rs	        return Err(MountError::RollbackTargetNotACandidate {\n            target,\n            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n        });	        return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_below_the_effective_floor	rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor
 增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-harness/src/crash_injection.rs	            for _ in 0..crash_points_per_history {	            for _ in 0..crash_points_per_history.max(1) {	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
+增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(&report.outcome);	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
 增补 3 第 3 件（用户 2026-09-20 定案第 1 条）：屏障不再切段（屏障进了枚举域，少一道屏障就把两段并成一段）	crates/singlefs-harness/src/crash.rs	            RecordedOperationKind::Barrier => {\n                if !current.is_empty() {\n                    segments.push(std::mem::take(&mut current));\n                }\n            }	            RecordedOperationKind::Barrier => {}	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- every_crash_state_of_a_written_out_history	every_crash_state_of_a_written_out_history_recovers_into_a_committed_version
 增补 3 第 3 件（用户 2026-09-20 定案第 2 条）：段内只截前缀，不摆任意真子集（「后发的写先持久」整类零覆盖）	crates/singlefs-harness/src/crash_injection.rs	    if segment_length <= WRITES_A_SUBSET_MASK_HOLDS {\n        let mask = source.below((1u64 << segment_length) - 1);\n        return bits_of(mask, segment_length);\n    }\n    let mut persisted: Vec<bool> = (0..segment_length).map(|_| source.below(2) == 1).collect();\n    if persisted.iter().all(|is_persisted| *is_persisted) {\n        let withheld = usize::try_from(source.below(u64::try_from(segment_length).expect("段长")))\n            .expect("下标装得进 usize");\n        persisted[withheld] = false;\n    }\n    persisted	    let persisted_prefix = source.below(u64::try_from(segment_length).expect("段长"));\n    (0..segment_length)\n        .map(|write| u64::try_from(write).expect("下标") < persisted_prefix)\n        .collect()	-p singlefs-harness --lib -- crash_injection::tests::crash_points_withhold_every_kind_of_write	crash_points_withhold_every_kind_of_write_and_leave_holes_inside_a_segment
 增补 3 第 3 件（用户 2026-09-20 定案第 3 条）：崩溃状态上不跑记录核对器	crates/singlefs-harness/src/crash_injection.rs	        let record_check =\n            check_records_against(&image, writes_up_to_this_segment, report.effective_root);\n        tally.record_checks += 1;	        let record_check = RecordCheck::default();	-p singlefs-harness --lib -- crash_injection::tests::crash_points_are_reproducible	crash_points_are_reproducible_proper_subsets_sorted_by_segment
```

### `crates/singlefs-core/src/write_accounting.rs`（原始 diff 第 55-67 行）

```diff
diff --git a/crates/singlefs-core/src/write_accounting.rs b/crates/singlefs-core/src/write_accounting.rs
index 466d4ed..1f713c8 100644
--- a/crates/singlefs-core/src/write_accounting.rs
+++ b/crates/singlefs-core/src/write_accounting.rs
@@ -75,7 +75,7 @@ impl WrittenStructureKind {
             WrittenStructureKind::InstanceTableUnit => "instance_table_unit",
             WrittenStructureKind::JournalRecord => "journal_record",
             WrittenStructureKind::RootSlot => "root_slot",
-            WrittenStructureKind::SystemConfigurationSlot => "superblock_slot",
+            WrittenStructureKind::SystemConfigurationSlot => "system_configuration_slot",
         }
     }
 }
```

### `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`（原始 diff 第 68-80 行）

```diff
diff --git a/crates/singlefs-harness/src/bin/first_transaction_on_device.rs b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
index e13dd64..f2b6299 100644
--- a/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
+++ b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
@@ -1080,7 +1080,7 @@ mod tests {
              instance_table_unit_write_calls=0 instance_table_unit_written_bytes=0 \
              journal_record_write_calls=2 journal_record_written_bytes=8192 \
              root_slot_write_calls=1 root_slot_written_bytes=512 \
-             superblock_slot_write_calls=0 superblock_slot_written_bytes=0"
+             system_configuration_slot_write_calls=0 system_configuration_slot_written_bytes=0"
         );
     }
 
```

### `crates/singlefs-harness/src/first_transaction_regions.rs`（原始 diff 第 81-100 行）

```diff
diff --git a/crates/singlefs-harness/src/first_transaction_regions.rs b/crates/singlefs-harness/src/first_transaction_regions.rs
index 4388169..626b697 100644
--- a/crates/singlefs-harness/src/first_transaction_regions.rs
+++ b/crates/singlefs-harness/src/first_transaction_regions.rs
@@ -165,13 +165,13 @@ pub const FIRST_TRANSACTION_REGIONS: [FirstTransactionRegion; FIRST_TRANSACTION_
         JOURNAL_RECORD_BYTES,
     ),
     fixed_structure_region(
-        "superblock",
+        "system_configuration",
         0,
         FIRST_TRANSACTION_SYSTEM_CONFIGURATION_SLOT_OFFSET,
         SYSTEM_CONFIGURATION_SLOT_BYTES,
     ),
     fixed_structure_region(
-        "superblock",
+        "system_configuration",
         1,
         FIRST_TRANSACTION_SYSTEM_CONFIGURATION_SLOT_OFFSET,
         SYSTEM_CONFIGURATION_SLOT_BYTES,
```

### `crates/singlefs-harness/src/lib.rs`（原始 diff 第 101-112 行）

```diff
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
index 2f892f4..d0b9307 100644
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -14,7 +14,6 @@ use singlefs_core::block_device::{
 };
 
 pub mod crash;
-pub mod crash_injection;
 pub mod device_log;
 pub mod first_transaction_regions;
 pub mod hexadecimal;
```

### `crates/singlefs-harness/src/segments.rs`（原始 diff 第 113-152 行）

```diff
diff --git a/crates/singlefs-harness/src/segments.rs b/crates/singlefs-harness/src/segments.rs
index 72431b4..256459d 100644
--- a/crates/singlefs-harness/src/segments.rs
+++ b/crates/singlefs-harness/src/segments.rs
@@ -28,7 +28,7 @@ impl StepKind {
             StepKind::UnitWrite => "unit_write",
             StepKind::JournalRecord => "journal_record",
             StepKind::RootRecordFua => "root_record_fua",
-            StepKind::SystemConfigurationSlot => "superblock_slot",
+            StepKind::SystemConfigurationSlot => "system_configuration_slot",
             StepKind::Barrier => "barrier",
         }
     }
@@ -230,7 +230,7 @@ mod tests {
         assert_eq!(segment_sizes_text(&segments), "2+1+2");
         assert_eq!(
             segment_kinds_text(&segments),
-            "[unit_write×2,barrier]|[root_record_fua]|[superblock_slot×2,barrier]"
+            "[unit_write×2,barrier]|[root_record_fua]|[system_configuration_slot×2,barrier]"
         );
         assert_eq!(closed_form_state_count(&segments), 1 + 3 + 1 + 3);
 
@@ -248,7 +248,7 @@ mod tests {
         assert_eq!(segment_sizes_text(&warm_up_segments), "2+1+2");
         assert_eq!(
             segment_kinds_text(&warm_up_segments),
-            "[journal_record×2,barrier×2]|[root_record_fua]|[superblock_slot×2]"
+            "[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]"
         );
 
         // 种类串按枚举声明序，不按出现序：系统配置槽写先于单元写发出，串里仍是 unit_write 在前。
@@ -260,7 +260,7 @@ mod tests {
         ];
         assert_eq!(
             segment_kinds_text(&split_into_segments(&mixed, &geometry)),
-            "[unit_write,superblock_slot×2,barrier]"
+            "[unit_write,system_configuration_slot×2,barrier]"
         );
 
         // FUA 不替它前面的普通写做持久：普通写与 FUA 同段。
```

### `crates/singlefs-harness/tests/first_transaction_region_bytes.rs`（原始 diff 第 153-167 行）

```diff
diff --git a/crates/singlefs-harness/tests/first_transaction_region_bytes.rs b/crates/singlefs-harness/tests/first_transaction_region_bytes.rs
index 289f674..e818340 100644
--- a/crates/singlefs-harness/tests/first_transaction_region_bytes.rs
+++ b/crates/singlefs-harness/tests/first_transaction_region_bytes.rs
@@ -126,8 +126,8 @@ fn registry_rows() -> Vec<(&'static str, u32, u64, u64, HexadecimalExtent)> {
         ("journal_record",  0, 16 * 1024 * 1024 + 8192, 4096, WholeRegion),
         ("journal_record",  1, 16 * 1024 * 1024 + 8192, 4096, WholeRegion),
         // t11：系统配置槽 1（世代号 5、tail = 3），槽距 4096、槽宽 4096
-        ("superblock",      0, 4096, 4096, WholeRegion),
-        ("superblock",      1, 4096, 4096, WholeRegion),
+        ("system_configuration",      0, 4096, 4096, WholeRegion),
+        ("system_configuration",      1, 4096, 4096, WholeRegion),
     ]
 }
 
```

### `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs`（原始 diff 第 168-216 行）

```diff
diff --git a/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs b/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
index b9e3606..9dd41e2 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
@@ -308,7 +308,7 @@ fn recorded_paths_match_the_registered_segment_sequences() {
     );
     assert_eq!(
         kinds(mkfs_operations),
-        "[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[superblock_slot×4,barrier]"
+        "[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier]"
     );
     assert_eq!(
         (
@@ -318,7 +318,7 @@ fn recorded_paths_match_the_registered_segment_sequences() {
         ),
         (2, "2".to_string(), 4)
     );
-    assert_eq!(kinds(acquisition_operations), "[superblock_slot×2]");
+    assert_eq!(kinds(acquisition_operations), "[system_configuration_slot×2]");
     assert_eq!(
         (
             warm_up_operations.len(),
@@ -329,7 +329,7 @@ fn recorded_paths_match_the_registered_segment_sequences() {
     );
     assert_eq!(
         kinds(warm_up_operations),
-        "[journal_record×2,barrier×2]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]"
+        "[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]"
     );
     assert_eq!(
         (
@@ -341,7 +341,7 @@ fn recorded_paths_match_the_registered_segment_sequences() {
     );
     assert_eq!(
         kinds(transaction_operations),
-        "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]"
+        "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]"
     );
     assert_eq!(
         (
@@ -353,7 +353,7 @@ fn recorded_paths_match_the_registered_segment_sequences() {
     );
     assert_eq!(
         kinds(post_mkfs_operations),
-        "[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]"
+        "[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]"
     );
     // 每一步恰好落在一个段里。
     for slice in [
```

### `crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs`（原始 diff 第 217-238 行）

```diff
diff --git a/crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs b/crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs
index aa756ae..389a393 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs
@@ -186,7 +186,7 @@ fn checker_reads_the_same_generation_zero_root_from_all_three_regions_and_isolat
 #[test]
 fn both_system_configuration_slots_verify_with_the_same_generation_and_a_corrupt_slot_loses_the_choice(
 ) {
-    let mut pool = run_mkfs("superblock");
+    let mut pool = run_mkfs("system_configuration");
     let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
     for (device_number, (_, device)) in pool.devices.iter().enumerate() {
         let slot_zero = read(device, 0, slot_bytes);
@@ -266,7 +266,7 @@ fn recorded_stream_matches_the_registered_mkfs_segment_sequence() {
     );
     assert_eq!(
         segment_kinds_text(&segments),
-        "[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[superblock_slot×4,barrier]"
+        "[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier]"
     );
     assert_eq!(operations.len(), 13, "mkfs 13 次操作（含两道池屏障）");
     assert_eq!(
```

### `crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs`（原始 diff 第 239-260 行）

```diff
diff --git a/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs b/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
index ca6e2e9..ac1b6cd 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
@@ -103,7 +103,7 @@ fn probes() -> Vec<Probe> {
             flips: both(tree_table_offset, 300),
         },
         Probe {
-            name: "superblock_slot_one_both_devices",
+            name: "system_configuration_slot_one_both_devices",
             flips: both(DeviceOffsetInBytes(4096), 50),
         },
     ]
@@ -185,7 +185,7 @@ fn probes_behave_as_milestone_step_six_expects() {
         ),
         // 陈旧的 tail（槽 1 坏了就择回槽 0：tail 2）：全环扫描不信 tail，结果与 tail 正确时相同。
         (
-            "superblock_slot_one_both_devices",
+            "system_configuration_slot_one_both_devices",
             file_read(ROOT_ONE_THREE),
             3,
             0,
```

### `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`（原始 diff 第 261-273 行）

```diff
diff --git a/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs b/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
index b440c0c..34a54c4 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
@@ -88,7 +88,7 @@ fn overwrite_publishes_the_second_version_through_the_same_commit_shape() {
     );
     assert_eq!(
         segment_kinds_text(&segments),
-        "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]"
+        "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]"
     );
 
     assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
```

### `research/e7-index-bench/src/bin/e100_system_configuration_slot.rs`（原始 diff 第 274-338 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e100_system_configuration_slot.rs b/research/e7-index-bench/src/bin/e100_system_configuration_slot.rs
index b87a14f..2df8dd9 100644
--- a/research/e7-index-bench/src/bin/e100_system_configuration_slot.rs
+++ b/research/e7-index-bench/src/bin/e100_system_configuration_slot.rs
@@ -1,23 +1,23 @@
-//! E100：超级块的三段几何 —— D22 已定项 9 欠的那次测量。
+//! E100：系统配置的三段几何 —— D22 已定项 9 欠的那次测量。
 //!
 //! ## 被引用条款逐字贴在这里
 //!
-//! - **D22 已定项 8**：超级块**每盘一份**，更新走 **≥2 槽轮换**。
+//! - **D22 已定项 8**：系统配置**每盘一份**，更新走 **≥2 槽轮换**。
 //! - **D22 已定项 21**：原地覆写的结构要带**整单元校验和**与**被实际检查的世代号**。
 //! - **D20 推论三**：撕裂判定宽度 = 运行时探测的 `physical_block_size`
 //!   ⇒ **一个槽要原子，就不能跨扇区**。本机 512（E79 用的就是它），另一档 4096。
-//! - **D15 逐字**：feature bit「**位数**：三个 bitmap 各 **256 位**（`u64[4]`），存超级块」
+//! - **D15 逐字**：feature bit「**位数**：三个 bitmap 各 **256 位**（`u64[4]`），存系统配置」
 //!   ⇒ 光 feature bit 就是 **96 字节**。
-//! - **D15 止损规则逐字**：「**第 1、2 层冻结前必须已包含 D9 全部四项预留**」，而超级块是第 1 层。
-//! - **D9 已定项 8 逐字**：「nonce 水位**不许住在裸明文超级块里**，必须落在一个被主密钥派生
-//!   MAC 覆盖的字段上」⇒ 超级块要给 MAC 与 nonce 水位留位。
-//! - **D12 已定项 3**：设备级几何量住超级块里的**设备描述符表**；**异构池要支持**。
+//! - **D15 止损规则逐字**：「**第 1、2 层冻结前必须已包含 D9 全部四项预留**」，而系统配置是第 1 层。
+//! - **D9 已定项 8 逐字**：「nonce 水位**不许住在裸明文系统配置里**，必须落在一个被主密钥派生
+//!   MAC 覆盖的字段上」⇒ 系统配置要给 MAC 与 nonce 水位留位。
+//! - **D12 已定项 3**：设备级几何量住系统配置里的**设备描述符表**；**异构池要支持**。
 //! - **E72**：表的字节数恰好 `设备数 × 条目宽度`；**条目宽度仓里没定过**，按 24 / 40 / 64 三档算。
-//! - **D23**：「**journal 几何进超级块**（环大小、最大记录字节、在飞记录数上限），**不许是代码常量**」。
-//! - **D16 已定项 5**：「`T_dirty` 是可调值，不是格式常量：超级块里那个字段是格式，
+//! - **D23**：「**journal 几何进系统配置**（环大小、最大记录字节、在飞记录数上限），**不许是代码常量**」。
+//! - **D16 已定项 5**：「`T_dirty` 是可调值，不是格式常量：系统配置里那个字段是格式，
 //!   字段里放什么数不是」。
 //! - **C78 / E79 的先例**：512 字节槽带 jsn 水位只装 **6 棵树**，7 棵常识树集恰好 **513 字节爆 1 字节**
-//!   ⇒ 根记录**恒走间接层**（D22 已定项 7）。**超级块面对的是同一个形状。**
+//!   ⇒ 根记录**恒走间接层**（D22 已定项 7）。**系统配置面对的是同一个形状。**
 //!
 //! ## 判据（E100 正文跑前写死，跑完不许改）
 //!
@@ -25,7 +25,7 @@
 //!    逐格报「装得下 / 爆多少字节」。
 //! 2. **设备表是唯一随规模长的东西**：扫设备数 1 / 2 / 8 / 64 / 256 × 条目宽 24 / 40 / 64，
 //!    **数出第一个爆槽的格子**。
-//! 3. **出路臂**：设备表移出槽（超级块只存指向设备表单元的指针，形态同 D22 已定项 7 的间接层）
+//! 3. **出路臂**：设备表移出槽（系统配置只存指向设备表单元的指针，形态同 D22 已定项 7 的间接层）
 //!    ⇒ 三段字节数必须变成**与设备数无关的常数**，且两档槽宽上恒装得下。
 //! 4. **原子性判据**：一个槽必须恰好一个扇区。超过时**数出跨扇区的槽在撕裂点上
 //!    有几种可读但不一致的状态**。
@@ -64,7 +64,7 @@ const SECTION_ONE_BOOTSTRAP_FIELDS: [(&str, u64); 7] = [
 ];
 
 /// D9 的 day-1 预留（D15 止损规则：第 1、2 层冻结前必须已包含 D9 全部四项预留）。
-const SECTION_ONE_CRYPTO_FIELDS: [(&str, u64); 2] = [("sb_mac", 16), ("nonce_watermark", 8)];
+const SECTION_ONE_CRYPTO_FIELDS: [(&str, u64); 2] = [("system_configuration_mac", 16), ("nonce_watermark", 8)];
 
 /// 段二：几何。设备表另算（它是唯一随规模长的）。
 const SECTION_TWO_FIXED_GEOMETRY_FIELDS: [(&str, u64); 8] = [
@@ -91,7 +91,7 @@ const SECTION_THREE_TUNABLE_FIELDS: [(&str, u64); 5] = [
 /// 第一版提案的三段轴收不了它们——tail 槽每个 checkpoint 改一次，
 /// 既非永不改、非 mkfs 只读、也非人调的可调值。
 const SECTION_FOUR_RUNTIME_FIELDS: [(&str, u64); 2] = [
-    ("journal_tail", 8),     // D23 已定项 3：tail 住超级块槽
+    ("journal_tail", 8),     // D23 已定项 3：tail 住系统配置槽
     ("journal_instance", 4), // D23 已定项 9 的实例代号
 ];
 
```

### `research/e7-index-bench/src/bin/e102_unit_class_registry.rs`（原始 diff 第 339-391 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e102_unit_class_registry.rs b/research/e7-index-bench/src/bin/e102_unit_class_registry.rs
index 6353137..7fbde40 100644
--- a/research/e7-index-bench/src/bin/e102_unit_class_registry.rs
+++ b/research/e7-index-bench/src/bin/e102_unit_class_registry.rs
@@ -9,18 +9,18 @@
 //!   可不可能逐指针不同』定，不能按『它今天是什么值』定」。
 //! - **D18 已定项 7**：共同前缀 42 = magic 4 + 版本 2 + flags 2（含单元类标签位）+ 声明长度 2 +
 //!   头校验和 32；数据单元类身份段 = 五元组 33（标签 1 + 树 8 + 对象 8 + 出生代 8 + 锚点 8）+
-//!   诞生代号 8 + fsid 8 ⇒ 头 91；「自证单元（journal 记录 / 根槽 / 超级块槽）不用这张表」。
+//!   诞生代号 8 + fsid 8 ⇒ 头 91；「自证单元（journal 记录 / 根槽 / 系统配置槽）不用这张表」。
 //! - **D18 已定项 8**：「单元类型标签取『墓碑』」。**D18 已定项 10**：墓碑打包共享单元、
 //!   按代际装载、回收粒度 = 单元；E83：(32768 − 91) / 56 = 583。E98：(32768 − 91) / 140 = 233。
 //! - **D21 已定项 4**：扩展点归属三类（数据单元 / 索引节点 / 自证单元）。
 //! - **I-6.2**：两张白名单（数据单元类 / 元数据类）。
 //! - **D15 判定表**：新增记录类型 ⇒ compat_ro；改变已有字段含义 ⇒ incompat。
-//! - **D23 已定项 1 下属「记录头的类型字段」小节 A 条**：「未知类型 ⇒ 拒绝挂载，由超级块 incompat 位承载」。
+//! - **D23 已定项 1 下属「记录头的类型字段」小节 A 条**：「未知类型 ⇒ 拒绝挂载，由系统配置 incompat 位承载」。
 //!
 //! ## 提案（`research/prompts/_d18-item11-unit-class-registry.md`）里被模型化的部分
 //!
 //! 登记表码 0 无效 / 1 数据单元 / 2 索引节点 / 3 打包记录单元 / 16 根记录 / 17 journal 记录 /
-//! 18 超级块槽，其余保留；标签 1 字节住共同前缀偏移 6、是 AAD 首字节；打包记录单元类身份段 51 字节
+//! 18 系统配置槽，其余保留；标签 1 字节住共同前缀偏移 6、是 AAD 首字节；打包记录单元类身份段 51 字节
 //! （含 4 字节自包含载荷校验和；2026-09-05 C113 定案再加 10 字节写序；2026-09-12 C288 ① 再加 4 字节出生序号）⇒ 头 107；一个单元只装一种记录类型 / 一个代际 / 一棵树，记录定宽；未登记单元类 ⇒ 拒收 + incompat，
 //! 未登记记录类型 ⇒ 容器可验、内容跳过、只读（compat_ro）。
 //!
@@ -92,7 +92,7 @@ const REGISTRY: [(u8, &str, Kind); 7] = [
     (3, "packed_record_unit", Kind::Content),
     (16, "root_record", Kind::SelfCertifying),
     (17, "journal_record", Kind::SelfCertifying),
-    (18, "superblock_slot", Kind::SelfCertifying),
+    (18, "system_configuration_slot", Kind::SelfCertifying),
 ];
 
 /// 仓里四处清单 + D18 已定项 8 用到的类名（来源, 名字）。
@@ -105,7 +105,7 @@ const USED_NAMES: [(&str, &str); 15] = [
     ("D18-item7", "index_node"),
     ("D18-item7", "journal_record"),
     ("D18-item7", "root_slot"),
-    ("D18-item7", "superblock_slot"),
+    ("D18-item7", "system_configuration_slot"),
     ("D21-item4", "data_unit"),
     ("D21-item4", "index_node"),
     ("D21-item4", "self_cert_unit"),
@@ -124,7 +124,7 @@ fn codes_for(name: &str) -> Vec<u8> {
         "metadata_class" => vec![2, 3],
         "root_record" | "root_slot" => vec![16],
         "journal_record" => vec![17],
-        "superblock_slot" => vec![18],
+        "system_configuration_slot" => vec![18],
         // D21 已定项 4 的「自证单元」罩三个自证码
         "self_cert_unit" => vec![16, 17, 18],
         _ => vec![],
```

### `research/e7-index-bench/src/bin/e106_stripe_member_table.rs`（原始 diff 第 392-404 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e106_stripe_member_table.rs b/research/e7-index-bench/src/bin/e106_stripe_member_table.rs
index 89326c2..0fb05c7 100644
--- a/research/e7-index-bench/src/bin/e106_stripe_member_table.rs
+++ b/research/e7-index-bench/src/bin/e106_stripe_member_table.rs
@@ -10,7 +10,7 @@
 //! | 乙   | parity 格自己的头（成员单元预留恒零区） | 无（parity 自证）|
 //! | 丙   | 哪儿都不住（今天的状态）| 不适用，阳性对照 |
 //!
-//! 臂名不用拉丁字母：`A2` 与 D9（加密） 的前提编号 A2（超级块是明文）撞名，doc-lint 判红。
+//! 臂名不用拉丁字母：`A2` 与 D9（加密） 的前提编号 A2（系统配置是明文）撞名，doc-lint 判红。
 //!
 //! ## 判据（跑前写死，2026-09-06）
 //!
```

### `research/e7-index-bench/src/bin/e107_stripe_table_wa.rs`（原始 diff 第 405-417 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e107_stripe_table_wa.rs b/research/e7-index-bench/src/bin/e107_stripe_table_wa.rs
index e46cbf2..07a6e34 100644
--- a/research/e7-index-bench/src/bin/e107_stripe_table_wa.rs
+++ b/research/e7-index-bench/src/bin/e107_stripe_table_wa.rs
@@ -58,7 +58,7 @@ const CONTAINER_INDEX_HEADER_BYTES: u64 = 76;
 const CONTAINER_INDEX_ENTRY_BYTES: u64 = 85;
 /// 全池容器总数，用来钉容器索引的高（E106 口径：16 TiB / 90% 填充）。
 const TOTAL_CONTAINERS: u64 = 828_789;
-/// `w` 的上界，超级块声明的常量（D2 已定项 6）。
+/// `w` 的上界，系统配置声明的常量（D2 已定项 6）。
 const STRIPE_WIDTH_MAXIMUM: u64 = 4;
 /// `w` 的硬下界（D2 已定项 6：零冗余的条带不许发出）。
 const STRIPE_WIDTH_MINIMUM: u64 = 2;
```

### `research/e7-index-bench/src/bin/e115_system_configuration_completeness.rs`（原始 diff 第 418-518 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e115_system_configuration_completeness.rs b/research/e7-index-bench/src/bin/e115_system_configuration_completeness.rs
index cf5f042..ffa4b16 100644
--- a/research/e7-index-bench/src/bin/e115_system_configuration_completeness.rs
+++ b/research/e7-index-bench/src/bin/e115_system_configuration_completeness.rs
@@ -1,26 +1,26 @@
-//! # E115 超级块字段表的完备性与字节预算
+//! # E115 系统配置字段表的完备性与字节预算
 //!
-//! 答 D22 已定项 9（超级块的字段表）的挡路条 ② 与 ③。
+//! 答 D22 已定项 9（系统配置的字段表）的挡路条 ② 与 ③。
 //!
 //! E100 的 24 字段候选表是**自上而下列出来的**；2026-09-07 一次机械普查发现**反方向也有洞**——
-//! 已定条款明文要求进超级块的字段，有若干条在那张表里根本没有行。
+//! 已定条款明文要求进系统配置的字段，有若干条在那张表里根本没有行。
 //!
 //! ## 被引用条款（逐字）
 //!
-//! - D2 已定项 6：「**上界 4 是超级块声明的常量**，与设备数无关」
-//! - D2 已定项 8：「`g` 是超级块声明的常量，初值 4」
+//! - D2 已定项 6：「**上界 4 是系统配置声明的常量**，与设备数无关」
+//! - D2 已定项 8：「`g` 是系统配置声明的常量，初值 4」
 //! - D2 已定项 7（2026-08-31 用户定案）：「归属不再按 `(r × P) mod devs` 算，**mkfs 时逐区域把设备身份写下来**」；
-//!   「身份与超级块同址，而超级块本来就要读」。E62 实测的字节代价逐字是「**4 字节**（4 区域 × 1 字节）」
+//!   「身份与系统配置同址，而系统配置本来就要读」。E62 实测的字节代价逐字是「**4 字节**（4 区域 × 1 字节）」
 //! - D19 已定项 4：位置条目 14 字节 = 设备 ID **4** + 物理偏移 6（16 KiB 槽号）+ 密文校验和 4
-//! - I-8.1：「`环大小 ≥ F × 任一事务的最坏 journal 占用`，F ≥ 2。**最坏占用与 F 写进超级块**」
-//! - I-6.6：「任一指针记录的 MAC 长度与**超级块声明的**一致」
-//! - D9 day-1 预留第 3 项：「超级块的 **KDF 标识 + 主密钥槽 + 加密类型**」
+//! - I-8.1：「`环大小 ≥ F × 任一事务的最坏 journal 占用`，F ≥ 2。**最坏占用与 F 写进系统配置**」
+//! - I-6.6：「任一指针记录的 MAC 长度与**系统配置声明的**一致」
+//! - D9 day-1 预留第 3 项：「系统配置的 **KDF 标识 + 主密钥槽 + 加密类型**」
 //! - D12 已定项 2：该三样归**必须共用**桶
 //! - D15 止损规则：「第 1、2 层冻结前必须已包含 D9 全部四项预留」
-//! - D21：「扩展点大小 N 由每条线在**超级块里声明**、可以取 0」
+//! - D21：「扩展点大小 N 由每条线在**系统配置里声明**、可以取 0」
 //! - D19 已定项 5：中央映射是解引用与释放判定的**唯一入口**
-//! - D22 已定项 2：「区域 r 落在 `r × P × chunk`，P 素数且 `P > devs`」；「S 住超级块」；R = 3
-//! - D20 推论三：自证单元清单逐字只有「**根槽、journal 记录头**」两类，**未列超级块槽**
+//! - D22 已定项 2：「区域 r 落在 `r × P × chunk`，P 素数且 `P > devs`」；「S 住系统配置」；R = 3
+//! - D20 推论三：自证单元清单逐字只有「**根槽、journal 记录头**」两类，**未列系统配置槽**
 //! - E100：四段 361 字节 + 间接层指针 59 = 420；跨 n 扇区有 `2ⁿ − 2` 种可读但不一致的状态
 //!
 //! ## 它答不了的
@@ -31,7 +31,7 @@
 use e7_index_bench::Emitter;
 
 /// D20 推论三：槽宽 = 探测到的 `physical_block_size`。
-/// ⚠️ **它的自证单元清单逐字不含超级块槽**——E100 把这两档套给超级块是类比延伸，不是条款覆盖。
+/// ⚠️ **它的自证单元清单逐字不含系统配置槽**——E100 把这两档套给系统配置是类比延伸，不是条款覆盖。
 const SLOT_WIDTHS: [u64; 2] = [512, 4096];
 
 /// D22 已定项 7 的树表单元指针宽度，间接层一个指针的口径。
@@ -54,7 +54,7 @@ const CANDIDATE: [(&str, u64); 24] = [
     ("slot_generation", 8),
     ("unit_checksum", 32),
     // D9 预留 24
-    ("sb_mac", 16),
+    ("system_configuration_mac", 16),
     ("nonce_watermark", 8),
     // 段二 几何 127
     ("node_bytes", 4),
@@ -76,7 +76,7 @@ const CANDIDATE: [(&str, u64); 24] = [
     ("journal_instance", 4),
 ];
 
-/// 已定条款要求进超级块的每一项。
+/// 已定条款要求进系统配置的每一项。
 /// 第三列 = 它在 `CANDIDATE` 里对应哪个字段；`None` 表示**候选表里没有这一行**。
 const REQUIRED: [(&str, &str, Option<&str>); 31] = [
     ("magic", "骨架", Some("magic")),
@@ -86,9 +86,9 @@ const REQUIRED: [(&str, &str, Option<&str>); 31] = [
     ("本盘 dev id", "D19 已定项 4", Some("this_dev_id")),
     ("槽世代号", "D22 已定项 21", Some("slot_generation")),
     ("整槽校验和", "D22 已定项 21", Some("unit_checksum")),
-    ("超级块自身 MAC", "D9 已定项 8", Some("sb_mac")),
+    ("系统配置自身 MAC", "D9 已定项 8", Some("system_configuration_mac")),
     ("nonce 水位", "D9 已定项 8", Some("nonce_watermark")),
-    ("节点大小", "D8「mkfs 把 16 KiB 写进超级块」", Some("node_bytes")),
+    ("节点大小", "D8「mkfs 把 16 KiB 写进系统配置」", Some("node_bytes")),
     ("单元大小", "D18 已定项 9", Some("unit_bytes")),
     ("落点粒度", "D3 已定项 7", Some("alloc_grain")),
     ("位置条目宽度", "D19 已定项 4", Some("posentry_widths")),
@@ -102,14 +102,14 @@ const REQUIRED: [(&str, &str, Option<&str>); 31] = [
     ("journal tail", "D23 已定项 3", Some("journal_tail")),
     ("journal 实例代号", "D23 已定项 9", Some("journal_instance")),
     // ↓ 以下十项，候选表里没有行
-    ("journal 最坏占用与 F", "I-8.1 逐字「写进超级块」", None),
+    ("journal 最坏占用与 F", "I-8.1 逐字「写进系统配置」", None),
     ("根环逐区域设备身份", "D2 已定项 7 用户定案", None),
-    ("条带宽度上界 w_max", "D2 已定项 6 逐字「超级块声明的常量」", None),
-    ("组大小 g", "D2 已定项 8 逐字「超级块声明的常量」", None),
+    ("条带宽度上界 w_max", "D2 已定项 6 逐字「系统配置声明的常量」", None),
+    ("组大小 g", "D2 已定项 8 逐字「系统配置声明的常量」", None),
     ("KDF 标识", "D9 day-1 预留 3 + D12 已定项 2", None),
     ("加密类型", "D9 day-1 预留 3 + D12 已定项 2", None),
     ("主密钥槽", "D9 day-1 预留 3 + D12 已定项 2", None),
-    ("MAC 长度声明", "I-6.6 逐字「与超级块声明的一致」", None),
+    ("MAC 长度声明", "I-6.6 逐字「与系统配置声明的一致」", None),
     ("设备数 devs", "D22 已定项 2「P > devs」；设备表移出后槽里无条目数", None),
 ];
 
```

### `research/e7-index-bench/src/bin/e124_system_configuration_recompute.rs`（原始 diff 第 519-546 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e124_system_configuration_recompute.rs b/research/e7-index-bench/src/bin/e124_system_configuration_recompute.rs
index b7a50f6..739bb36 100644
--- a/research/e7-index-bench/src/bin/e124_system_configuration_recompute.rs
+++ b/research/e7-index-bench/src/bin/e124_system_configuration_recompute.rs
@@ -1,4 +1,4 @@
-//! # E124 超级块字段表按 D18 已定项 14 重算
+//! # E124 系统配置字段表按 D18 已定项 14 重算
 //!
 //! 答 D22 已定项 9 那条 2026-09-08 复核留下的账。该段在「D18 十四条全部定案」之后逐字写：
 //! 「**已定项 14（nonce 代号就是完整的 96 位 nonce，住单元明文头）改了挡路条 ② 那一格的输入**
@@ -11,7 +11,7 @@
 //! ## 被引用条款（逐字）
 //!
 //! - D18 已定项 14：「**取臂甲**：nonce 代号就是完整的 96 位 nonce（12 字节），住单元明文头，指针里那份保留」
-//! - D9 已定项 8：「nonce 水位**不许住在裸明文超级块里**，必须落在一个被主密钥派生 MAC 覆盖的字段上」
+//! - D9 已定项 8：「nonce 水位**不许住在裸明文系统配置里**，必须落在一个被主密钥派生 MAC 覆盖的字段上」
 //! - I-6.5：「记录在案的 nonce 水位 > 全盘出现过的最大 nonce（崩溃后不重用的可判定形式）」
 //! - D18 已定项 11 码 3 字段表：容器头偏移 69 那一行逐字 `| 69 | 记录数 | 2 | ≤ 583 |`
 //! - D22 已定项 7：树表单元指针 **59** = 指针头部 31 + 位置条目 14 × 2
@@ -60,7 +60,7 @@ const CANDIDATE_WITHOUT_WATERMARK_ROWS: [(&str, u64); 23] = [
     ("this_dev_id", 4),
     ("slot_generation", 8),
     ("unit_checksum", 32),
-    ("sb_mac", 16),
+    ("system_configuration_mac", 16),
     // ("nonce_watermark", ?) —— 判据 1 的被测量，见 `slot_bytes` 的 watermark_bytes 参数
     // 段二 几何
     ("node_bytes", 4),
```

### `research/e7-index-bench/src/bin/e126_system_configuration_slot_width.rs`（原始 diff 第 547-606 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e126_system_configuration_slot_width.rs b/research/e7-index-bench/src/bin/e126_system_configuration_slot_width.rs
index b5ed1b4..4238632 100644
--- a/research/e7-index-bench/src/bin/e126_system_configuration_slot_width.rs
+++ b/research/e7-index-bench/src/bin/e126_system_configuration_slot_width.rs
@@ -1,20 +1,20 @@
-//! # E126 超级块槽宽四条候选的代价
+//! # E126 系统配置槽宽四条候选的代价
 //!
-//! 答 C232 挡着的那一格：超级块槽宽取哪一条候选。**只给代价，不给判决**——
+//! 答 C232 挡着的那一格：系统配置槽宽取哪一条候选。**只给代价，不给判决**——
 //! 判决要走三方（.claude/rules/three-way-inference.md）。判据与失败条款见
 //! `.claude/kb/experiments/126-系统配置槽宽四条候选的代价.md`，**跑前写死**。
 //!
 //! ## 为什么这一格是空的（跑前现查，逐字）
 //!
 //! - `.claude/kb/decisions/18-块里携带什么信息.md` 第 775 行：
-//!   `| 18 | 超级块槽 | 槽宽 | D22 已定项 9 | 自己的 | 占位，未定 | 不适用 | 不带 | 同上 |`
+//!   `| 18 | 系统配置槽 | 槽宽 | D22 已定项 9 | 自己的 | 占位，未定 | 不适用 | 不带 | 同上 |`
 //!   ⇒ 第一个事务的字段表把它标成未定，且指回 D22 已定项 9 自己（循环）。
 //! - D20 推论三第 69 行：「**自证单元**（根槽、journal 记录头） | **等于运行时探测到的
-//!   `physical_block_size`，不许硬编码。**」⇒ **清单不含超级块槽。**
-//! - `e115_system_configuration_completeness.rs` 第 33–34 行自陈：「⚠️ **它的自证单元清单逐字不含超级块槽**
-//!   ——E100 把这两档套给超级块是类比延伸，不是条款覆盖。」
-//! - C212 逐字：「超级块槽宽跟着根槽走」，而同一格又逐字「**三条候选互不相同，不许预设走哪一条**」。
-//! - D22 已定项 8：「超级块每盘放一份……做成槽轮换，每盘至少 2 个槽。」**一个字没说多宽。**
+//!   `physical_block_size`，不许硬编码。**」⇒ **清单不含系统配置槽。**
+//! - `e115_system_configuration_completeness.rs` 第 33–34 行自陈：「⚠️ **它的自证单元清单逐字不含系统配置槽**
+//!   ——E100 把这两档套给系统配置是类比延伸，不是条款覆盖。」
+//! - C212 逐字：「系统配置槽宽跟着根槽走」，而同一格又逐字「**三条候选互不相同，不许预设走哪一条**」。
+//! - D22 已定项 8：「系统配置每盘放一份……做成槽轮换，每盘至少 2 个槽。」**一个字没说多宽。**
 //!
 //! ## 被引用条款（逐字）
 //!
@@ -32,7 +32,7 @@
 //!
 //! ## 它答不了的
 //!
-//! 不答「超级块该有哪些字段」（D22 已定项 9）——记录字节数是**输入**，按六格扫。
+//! 不答「系统配置该有哪些字段」（D22 已定项 9）——记录字节数是**输入**，按六格扫。
 //! 不答 C212 整条冲突（还管根槽与 journal 记录两类）。
 //! 不答真设备行为——`io_min` 三档是扫的参数。
 
@@ -56,7 +56,7 @@ enum Arm {
     ProbeLargerOfPhysicalBlockSizeAndMinimumInputOutputSize,
     /// 丙：= 一个格式常量（与探测解绑）
     FormatConstant(u64),
-    /// 丁：= 池内最小单元大小 ⇒ 超级块槽就是一个单元
+    /// 丁：= 池内最小单元大小 ⇒ 系统配置槽就是一个单元
     MinimumUnit,
 }
 
@@ -100,7 +100,7 @@ fn honors_minimum_input_output_size(width: u64, io_min: u64) -> bool {
 }
 
 /// 判据 4：跨几个 512 扇区，以及「可读但不一致」的态数 2ⁿ − 2。
-/// ⚠️ **暴露面不是损失面**（E34（根环槽几何） 逐字）：超级块无父、走 D22 已定项 21 的自证校验和，
+/// ⚠️ **暴露面不是损失面**（E34（根环槽几何） 逐字）：系统配置无父、走 D22 已定项 21 的自证校验和，
 /// 撕裂由整槽校验和抓；这个数量的是「有多少种可读的中间态」，不是「会丢多少」。
 /// 返回 `(跨扇区数 n, 2ⁿ − 2 的精确值)`；n > 63 时 u64 装不下，第二项返回 `None`
 /// ——**报 n 本身就够**，它才是暴露面的绝对值。
```

### `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`（原始 diff 第 607-1046 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs b/research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs
index f80cad6..aa5b2d2 100644
--- a/research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs
+++ b/research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs
@@ -79,7 +79,7 @@ const JOURNAL_HEADER_TEN_FIELD_BYTES: u64 = 78;
 const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 2 * NODE_POINTER_BYTES + 8 + 8;
 /// D23（journal 的角色与格式） 已定项 4 口径的点名项宽度；构成无落点，装置按字节表六的预想构成写。
 const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
-/// 超级块字段表合计（D22（单元原子性怎么合成） 已定项 9 + 已定项 15）：2026-09-14 用户定案加四个字段——
+/// 系统配置字段表合计（D22（单元原子性怎么合成） 已定项 9 + 已定项 15）：2026-09-14 用户定案加四个字段——
 /// 自举头的写入者身份 20 与校验和算法标识 1、几何段的 mkfs 时 physical_block_size 4 与扩展点声明值 N 4，共 29。
 const SYSTEM_CONFIGURATION_BYTES: u64 = 481;
 /// 头校验和 / 自证校验和的字段宽度（D18 已定项 7、D22 已定项 7、D23 已定项 4 同口径）。
@@ -95,12 +95,12 @@ const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 86;
 /// 码 2 头里 key 宽那一格的偏移（D18 已定项 18）：共同前缀 42 + 树 ID 8 + 层级 1 = 51。
 const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
 
-/// 字节表零：超级块槽 0 / 1 的设备内偏移。槽距 = 固定结构槽距 4096，与槽宽同值 ⇒ 两个槽首尾相接。
+/// 字节表零：系统配置槽 0 / 1 的设备内偏移。槽距 = 固定结构槽距 4096，与槽宽同值 ⇒ 两个槽首尾相接。
 const SYSTEM_CONFIGURATION_SLOT_OFFSETS: [u64; 2] = [0, 4096];
 /// D22（单元原子性怎么合成） 已定项 2 的槽宽那一格（2026-09-14 三方论证后按主 agent 推荐值写）：
-/// **超级块槽宽是格式常量 4096**，不再等于挂载时探测到的 `physical_block_size`。
+/// **系统配置槽宽是格式常量 4096**，不再等于挂载时探测到的 `physical_block_size`。
 /// 整槽校验和罩这 4096 字节含补齐（D18（块里携带什么信息） 已定项 17），481 字节的字段表在槽里余 3615。
-/// ⚠️ 它只管超级块：根槽仍按判定宽度 512 写（字节表七）。
+/// ⚠️ 它只管系统配置：根槽仍按判定宽度 512 写（字节表七）。
 const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
 /// 字节表零：根环起点 1 MiB（槽 64）、P = 3、chunk = 1 MiB、每区 8 槽、槽距 4096（预想）。
 const RING_START_OFFSET: u64 = 1 << 20;
@@ -120,10 +120,10 @@ const JOURNAL_RING_BYTES: u64 = 768 << 20;
 const JOURNAL_RING_SLOTS: u64 = JOURNAL_RING_BYTES / JOURNAL_RECORD_BYTES;
 const JOURNAL_SAFETY_FACTOR: u64 = 3;
 const JOURNAL_IN_FLIGHT_RECORD_LIMIT: u64 = JOURNAL_RING_SLOTS / JOURNAL_SAFETY_FACTOR;
-/// 超级块几何段的「journal 最坏占用」：在飞记录数上限 × 一条记录的字节数（I-8.1（环几何够大））。
+/// 系统配置几何段的「journal 最坏占用」：在飞记录数上限 × 一条记录的字节数（I-8.1（环几何够大））。
 const JOURNAL_WORST_CASE_BYTES: u64 = JOURNAL_IN_FLIGHT_RECORD_LIMIT * JOURNAL_RECORD_BYTES;
 
-/// D3（空间分配） 已定项 10 ④：单元区起始槽号进超级块，第一版 = journal 环末尾的下一个槽（16 MiB + 768 MiB = 784 MiB）。
+/// D3（空间分配） 已定项 10 ④：单元区起始槽号进系统配置，第一版 = journal 环末尾的下一个槽（16 MiB + 768 MiB = 784 MiB）。
 const UNIT_AREA_START_SLOT: u64 = JOURNAL_START_SLOT + JOURNAL_RING_BYTES / SLOT_BYTES;
 /// 镜像大小是 mkfs 参数（跟 fsid、写入时刻同一类），装置取 4 GiB 并在 `name=config` 里报出来。
 /// 下界由 D23（journal 的角色与格式） 已定项 19 ③ 的「环 ≤ 设备容量 ÷ 4」逼出：默认 768 MiB 的环要 3 GiB 以上的盘。
@@ -214,8 +214,8 @@ const POINTER_COMPRESSED_LENGTH_NONE: u16 = 0;
 /// inode 树条目写自己（12）、extent 树条目写它服务的那个头（12），分配记录 / 记账 / livelist / 旁表 / deadlist 写 0。
 const TREE_TABLE_HEAD_IDENTIFIER_NONE: u64 = 0;
 
-/// D22（单元原子性怎么合成） 已定项 9（2026-09-14 用户定案）：超级块自举头的「写入者身份」=
-/// 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节；第一版写 `singlefs-rs` 与 1（I-1.5（超级块记管道身份）、D17（实现分层与第三方管道） 债 3）。
+/// D22（单元原子性怎么合成） 已定项 9（2026-09-14 用户定案）：系统配置自举头的「写入者身份」=
+/// 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节；第一版写 `singlefs-rs` 与 1（I-1.5（系统配置记管道身份）、D17（实现分层与第三方管道） 债 3）。
 const WRITER_IDENTITY_NAME: &[u8] = b"singlefs-rs";
 const WRITER_IDENTITY_NAME_BYTES: usize = 16;
 const WRITER_IDENTITY_VERSION: u32 = 1;
@@ -244,10 +244,10 @@ const FIXED_FSID: [u8; 16] = [0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x
 const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
 const FIRST_INODE_NUMBER: u64 = 1;
 /// D23（journal 的角色与格式） 已定项 16：mkfs 写实例代号 0（「mkfs、尚无实例」，不是有效实例），
-/// 第一次可写挂载取 max(超级块, 根环) + 1 = 1，并先写进每一份超级块之后才动单元。
+/// 第一次可写挂载取 max(系统配置, 根环) + 1 = 1，并先写进每一份系统配置之后才动单元。
 const MKFS_INSTANCE_GENERATION: u32 = 0;
 const FIRST_INSTANCE_GENERATION: u32 = 1;
-/// D22（单元原子性怎么合成） 已定项 16：超级块槽世代号从 1 起、每写一次 +1，写世代号 g 的那一次落在槽 `g mod 2`。
+/// D22（单元原子性怎么合成） 已定项 16：系统配置槽世代号从 1 起、每写一次 +1，写世代号 g 的那一次落在槽 `g mod 2`。
 const SYSTEM_CONFIGURATION_GENERATION_AT_MKFS: u64 = 1;
 const SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION: u64 = 2;
 /// D16 已定项 8（暖机取甲′，2026-09-13 用户定案）：mkfs 之后第一次可写挂载先连推空发布，直到本实例写成的根覆盖两块盘；
@@ -421,7 +421,7 @@ fn castagnoli_crc32(bytes: &[u8]) -> u32 {
 
 /// 「校验和字段自身按 0 参与」（I-2.4）：把 `[field_offset, field_offset + 32)` 清零后对 `[0, cover_end)` 求校验和。
 /// D18（块里携带什么信息） 已定项 17（2026-09-13 用户定案）：32 字节的校验和字段里放 CRC32C 4 字节 + 28 字节零，
-/// 根记录自证校验和、超级块整槽校验和、journal `header_csum` 同口径——此前装置取 SHA-256，那是 gap G5，已收口。
+/// 根记录自证校验和、系统配置整槽校验和、journal `header_csum` 同口径——此前装置取 SHA-256，那是 gap G5，已收口。
 fn wide_checksum_with_field_zeroed(bytes: &[u8], cover_end: usize, field_offset: usize) -> [u8; 32] {
     let mut covered = bytes[..cover_end].to_vec();
     covered[field_offset..field_offset + WIDE_CHECKSUM_BYTES as usize].fill(0);
@@ -639,7 +639,7 @@ enum StepKind {
     JournalRecord,
     /// 根环槽的一次 FUA 写（D16（发布语义） 已定项 7：只有这一步等落盘才发下一条）。
     RootRecordFua,
-    /// 超级块槽的一次写。
+    /// 系统配置槽的一次写。
     SystemConfigurationSlot,
 }
 
@@ -650,10 +650,10 @@ impl StepKind {
             StepKind::UnitWrite => "unit_write",
             StepKind::JournalRecord => "journal_record",
             StepKind::RootRecordFua => "root_record_fua",
-            StepKind::SystemConfigurationSlot => "superblock_slot",
+            StepKind::SystemConfigurationSlot => "system_configuration_slot",
         }
     }
-    /// FUA 由步骤种类决定，不再是调用点各传各的布尔：超级块槽写不可能是 FUA，这样它写不出来。
+    /// FUA 由步骤种类决定，不再是调用点各传各的布尔：系统配置槽写不可能是 FUA，这样它写不出来。
     fn is_fua(self) -> bool {
         match self {
             StepKind::RootRecordFua => true,
@@ -924,7 +924,7 @@ fn header_checksum_holds(bytes: &[u8], header_end: usize) -> bool {
     wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET) == bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32]
 }
 
-/// fsid 在单元头里是 8 字节（D18 已定项 7）；字节表二预想取超级块 fsid 的低 8 字节。
+/// fsid 在单元头里是 8 字节（D18 已定项 7）；字节表二预想取系统配置 fsid 的低 8 字节。
 fn unit_fsid(fsid: &[u8; 16]) -> u64 {
     u64::from_le_bytes(fsid[..8].try_into().expect("切了 8 字节"))
 }
@@ -1256,7 +1256,7 @@ fn data_unit_payload(bytes: &[u8], declared_length: u16) -> &[u8] {
     &bytes[start..start + declared_length as usize]
 }
 
-// ───────────────────────── 根记录、超级块、journal 记录 ─────────────────────────
+// ───────────────────────── 根记录、系统配置、journal 记录 ─────────────────────────
 
 /// D22（单元原子性怎么合成） 已定项 7 的字段表，371 字节，字段序照那张表（gap G4：字节表七的行序与它不同，两处都没写偏移）。
 #[derive(Clone, Copy, PartialEq, Eq, Debug)]
@@ -1326,7 +1326,7 @@ impl RootRecord {
     }
 }
 
-/// 超级块字段表（D22 已定项 9 + 已定项 15）：**481 字节**，512 槽内余 31。
+/// 系统配置字段表（D22 已定项 9 + 已定项 15）：**481 字节**，512 槽内余 31。
 /// 2026-09-13 用户定案去掉「间接目录单元指针 59」（第一版显式留白，不是留位），几何段加三个字段：
 /// 单元区起始槽号 8（D3 已定项 10 ④）、mkfs 时的 io_min 4 与固定结构槽距 4（D2 已定项 19）；
 /// 2026-09-14 用户定案再加四个：自举头的写入者身份 20 与校验和算法标识 1、
@@ -1348,10 +1348,10 @@ const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 4 + 2 + 96 + 16 + 20 + 1 + 4
 const SYSTEM_CONFIGURATION_FSID_OFFSET: usize = 4 + 2 + 96;
 const SYSTEM_CONFIGURATION_REGION_DEVICES_OFFSET: usize = 379;
 const SYSTEM_CONFIGURATION_TAIL_OFFSET: usize = 469;
-/// D2（RAID 条带策略） 已定项 19：固定结构槽距 = max(4096, mkfs 时探测的 io_min)，两个数各占超级块一个 4 字节字段。
+/// D2（RAID 条带策略） 已定项 19：固定结构槽距 = max(4096, mkfs 时探测的 io_min)，两个数各占系统配置一个 4 字节字段。
 const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
 const MKFS_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
-/// D2（RAID 条带策略） 已定项 18：第一版超级块里 w_max 与 g 都写 4；g 挂载时按可写设备数夹取。
+/// D2（RAID 条带策略） 已定项 18：第一版系统配置里 w_max 与 g 都写 4；g 挂载时按可写设备数夹取。
 const SYSTEM_CONFIGURATION_MAXIMUM_WIDTH: u8 = 4;
 const SYSTEM_CONFIGURATION_GROUP_SIZE: u8 = 4;
 
@@ -1362,7 +1362,7 @@ impl SystemConfiguration {
         writer.put_u16(FORMAT_VERSION);
         writer.put_u8(INCOMPAT_FIRST_SSD_LINE_BIT); // feature bits：incompat 位 0 = 第一条纯 SSD 布局线（D15 已定项 4，2026-09-13 用户定案）
         writer.skip(95); // 其余 incompat 位与 compat_ro / compat 两张位图全 0
-        writer.assert_position(SYSTEM_CONFIGURATION_FSID_OFFSET as u64, "超级块 fsid");
+        writer.assert_position(SYSTEM_CONFIGURATION_FSID_OFFSET as u64, "系统配置 fsid");
         writer.put(&self.fsid);
         // 写入者身份（D22 已定项 9，2026-09-14 用户定案）：实现标识 16 字节 ASCII 零补齐 + 版本 4 字节。
         let mut writer_identity = [0u8; WRITER_IDENTITY_NAME_BYTES];
@@ -1373,9 +1373,9 @@ impl SystemConfiguration {
         writer.put_u32(self.this_device.0);
         writer.put_u32(self.device_count);
         writer.put_u64(self.slot_generation);
-        writer.assert_position(SYSTEM_CONFIGURATION_CHECKSUM_OFFSET as u64, "超级块整槽校验和");
+        writer.assert_position(SYSTEM_CONFIGURATION_CHECKSUM_OFFSET as u64, "系统配置整槽校验和");
         writer.skip(WIDE_CHECKSUM_BYTES as usize);
-        writer.skip(16 + 12 + 4); // 超级块 MAC、nonce 水位、KDF 标识（4，D22 已定项 9）
+        writer.skip(16 + 12 + 4); // 系统配置 MAC、nonce 水位、KDF 标识（4，D22 已定项 9）
         writer.put_u8(0); // 加密类型：关
         writer.put_u8(16); // MAC 长度声明
         writer.skip(80); // 主密钥槽：内联进槽，加密关时全 0（D22 已定项 9，2026-09-13 用户定案）
@@ -1417,7 +1417,7 @@ impl SystemConfiguration {
         writer.assert_position(SYSTEM_CONFIGURATION_TAIL_OFFSET as u64, "journal tail");
         writer.put_u64(self.journal_tail);
         writer.put_u32(self.journal_instance.0);
-        writer.assert_position(SYSTEM_CONFIGURATION_BYTES, "超级块");
+        writer.assert_position(SYSTEM_CONFIGURATION_BYTES, "系统配置");
         let mut bytes = writer.bytes;
         // 「整槽校验和」：覆盖整个 4096 槽含补齐、自身按 0 参与（D18 已定项 17）。
         let digest = wide_checksum_with_field_zeroed(&bytes, SYSTEM_CONFIGURATION_SLOT_BYTES as usize, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
@@ -1543,7 +1543,7 @@ struct JournalRecord {
     transaction: TransactionNumber,
     is_commit: bool,
     back_chain: u32,
-    /// D23（journal 的角色与格式） 已定项 4（2026-09-14 用户定案）：超级块 fsid 的低 8 字节，与单元头同口径（I-1.4（块头 fsid 一致））。
+    /// D23（journal 的角色与格式） 已定项 4（2026-09-14 用户定案）：系统配置 fsid 的低 8 字节，与单元头同口径（I-1.4（块头 fsid 一致））。
     fsid: u64,
     /// D23（journal 的角色与格式） 已定项 15 的新根段：崩在记录持久之后、根槽持久之前时由它重建那次发布的根。
     new_tree_table: NodePointer,
@@ -2108,8 +2108,8 @@ fn mkfs(parameters: &PoolParameters) -> (RecordingPool, MkfsOutput) {
     (pool, MkfsOutput { root, instance_table_unit, tree_table_genesis_unit })
 }
 
-/// D23（journal 的角色与格式） 已定项 16：第一次可写挂载取 max(超级块, 根环) + 1 = 1，
-/// **并写进每一份超级块（一次超级块槽写，世代号 +1）之后才动单元**——所以它自成一段，排在暖机之前。
+/// D23（journal 的角色与格式） 已定项 16：第一次可写挂载取 max(系统配置, 根环) + 1 = 1，
+/// **并写进每一份系统配置（一次系统配置槽写，世代号 +1）之后才动单元**——所以它自成一段，排在暖机之前。
 /// 段的收尾靠暖机第一次空发布开头那道屏障（D16 已定项 7 的形态，不另加屏障：这是最少屏障的写法）。
 fn acquire_instance(pool: &mut RecordingPool, parameters: &PoolParameters) -> InstanceGeneration {
     let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
@@ -2260,13 +2260,13 @@ fn journal_record_offset(counter: JournalCounter) -> DeviceOffset {
     DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES + ((counter.0 - 1) % JOURNAL_RING_SLOTS) * JOURNAL_RECORD_BYTES)
 }
 
-/// 发布之后那次超级块槽写的世代号与落点（D22 已定项 16）：取号那次是 2，之后每次发布 +1，槽 = 世代号 mod 2。
+/// 发布之后那次系统配置槽写的世代号与落点（D22 已定项 16）：取号那次是 2，之后每次发布 +1，槽 = 世代号 mod 2。
 fn system_configuration_write_for_publish(checkpoint_txg: CheckpointTxg) -> (u64, usize) {
     let generation = checkpoint_txg.0 + SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION;
     (generation, (generation % 2) as usize)
 }
 
-/// 暖机（D16 已定项 8）：每次空发布照 D16 已定项 7 的顺序——屏障 → 空记录 → 屏障 → 根槽 FUA → 超级块槽轮换；
+/// 暖机（D16 已定项 8）：每次空发布照 D16 已定项 7 的顺序——屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换；
 /// 空记录不点名任何单元、事务号 0（D23 已定项 19 ①：0 保留给不承载事务的记录）、提交标记 1，
 /// 新根段照 mkfs 的根，根记录只改 checkpoint_txg。返回最后一条记录的字节，下一条记录的反向链要用它。
 fn warm_up(pool: &mut RecordingPool, parameters: &PoolParameters, genesis: &MkfsOutput, instance: InstanceGeneration) -> (Vec<RootRecord>, Option<Vec<u8>>) {
@@ -2634,7 +2634,7 @@ fn publish_first_file(
     let (region, ring_slot) = ring_target_for_publish(txg);
     pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, ring_slot), &root.to_slot(), StepKind::RootRecordFua);
 
-    // 根槽之后：超级块槽轮换，世代号 5、落槽 1（D22 已定项 16），tail 前移到 jsn 3（D16 已定项 7 的超级块注）。
+    // 根槽之后：系统配置槽轮换，世代号 5、落槽 1（D22 已定项 16），tail 前移到 jsn 3（D16 已定项 7 的系统配置注）。
     let (slot_generation, slot_index) = system_configuration_write_for_publish(txg);
     for device in parameters.devices() {
         let system_configuration = SystemConfiguration {
@@ -2726,13 +2726,13 @@ fn choose_system_configuration(reader: &dyn BlockReader) -> Result<SystemConfigu
             }
         }
         let Some(best_on_device) = best_on_device else {
-            return Err(format!("盘 {device_index} 两个超级块槽都无效"));
+            return Err(format!("盘 {device_index} 两个系统配置槽都无效"));
         };
         match &chosen {
             None => chosen = Some(best_on_device),
             Some(previous) => {
                 if previous.fsid != best_on_device.fsid || previous.device_count != best_on_device.device_count {
-                    return Err("两盘的超级块 fsid 或设备数对不上".to_string());
+                    return Err("两盘的系统配置 fsid 或设备数对不上".to_string());
                 }
             }
         }
@@ -3276,7 +3276,7 @@ fn probes(parameters: &PoolParameters) -> Vec<Probe> {
         Probe { name: "data_payload_both_copies", flips: both(data_offset, 200) },
         Probe { name: "data_header_last_byte_both_copies", flips: both(data_offset, DATA_UNIT_HEADER_BYTES - 1) },
         Probe { name: "tree_table_both_copies", flips: both(tree_table_offset, 300) },
-        Probe { name: "superblock_slot_one_both_devices", flips: both(DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1]), 50) },
+        Probe { name: "system_configuration_slot_one_both_devices", flips: both(DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1]), 50) },
     ]
 }
 
@@ -3350,8 +3350,8 @@ fn hex_bytes(bytes: &[u8]) -> String {
     bytes.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join("")
 }
 
-/// 量 1/3/4 的「所属结构」候选表：第一个事务写到的 8 个单元、根记录、journal 记录（第一节第 3 条的区域清单，去掉超级块——
-/// 超级块与改动计数无关，按第三节推导；它若意外出现在差异里，classify_offset 找不到候选、归到「未登记」而不是被这张表悄悄吃掉）。
+/// 量 1/3/4 的「所属结构」候选表：第一个事务写到的 8 个单元、根记录、journal 记录（第一节第 3 条的区域清单，去掉系统配置——
+/// 系统配置与改动计数无关，按第三节推导；它若意外出现在差异里，classify_offset 找不到候选、归到「未登记」而不是被这张表悄悄吃掉）。
 struct StructureCatalog<'output> {
     units: &'output [(SlotNumber, TransactionUnit, Vec<u8>)],
     root_device: u32,
@@ -3668,11 +3668,11 @@ const GAPS: &[(&str, &str)] = &[
     ("G18", "D23已定项12按「12项事务恰占1条记录」算余量，而D16的事务切分纪律让一次带8个数据单元的fsync至少是8个事务、8条记录；两条已定条款对同一负载算出的记录数不同"),
     ("G19", "mkfs种根的13次操作（11写+2屏障，段序列4+1+1+1+4、34个崩溃状态）不在层0枚举里：装置从mkfs之后的池起枚举（取号、暖机与事务），mkfs的崩溃状态没有任何东西判；段序列另发一行钉住"),
     ("G20", "已收口（2026-09-14用户定案，C321还清）：映射条目回到一宽55⇒码2头那个u16的条目宽字段够用，声明长度=条目数×条目宽原样成立；装置删掉了「条目宽0当变长哨兵」那套自创取法，parse_index_node对条目宽0一律判结构错"),
-    ("G21", "取号那一步（D23已定项16：第一次可写挂载写每一份超级块之后才动单元）不是根槽写路径，layout/01-first-txn八那张表罩不到它；屏障怎么放没有条款，装置按最少屏障取「不另加屏障，靠暖机第一次空发布开头那道屏障收段」⇒段序列独占一行[superblock_slot×2]"),
+    ("G21", "取号那一步（D23已定项16：第一次可写挂载写每一份系统配置之后才动单元）不是根槽写路径，layout/01-first-txn八那张表罩不到它；屏障怎么放没有条款，装置按最少屏障取「不另加屏障，靠暖机第一次空发布开头那道屏障收段」⇒段序列独占一行[system_configuration_slot×2]"),
     ("G22", "**仍欠着**（C323，2026-09-14加注）：镜像大小（单元区的末端）全仓仍没有条款，D23已定项19③只定了「环≤设备容量÷4」⇒它给出容量的**下界**而不是值；装置按mkfs参数取4GiB（1GiB装不下默认768MiB的环）并在name=config里报出来，空闲字节3472670720、全空聚簇段数3310、runs4三个数都随它变"),
     ("G23", "已收口（2026-09-14用户定案，C324还清）：码3打包容器按「数据单元」那一档取落点（起点32768对齐、两槽都空）；连同D3已定项10⑤的bump次序（树ID升序、树内先叶后根、映射树倒数第二、树表最末）⇒t2 extent根拿50240、游标停在50241而t3要对齐⇒t3拿50242–50243，**槽50241空着**、runs因此从3变4"),
     ("G24", "树表条目的「头ID」（D5已定项9，2026-09-14用户定案）只说了inode树写自己、extent树写12、其余写0，没说**这个数从哪来**：第一版只有一个可写头，装置按「inode树的树ID就是头ID」写；多头之后头ID与树ID还是不是一回事，条款没答"),
-    ("G25", "journal记录头2026-09-14加的fsid8与MAC16，条款只说fsid「与单元头同口径」（超级块fsid的低8字节）、MAC第一版全0；**读者拿fsid做什么没写**——装置按I-1.4把fsid不符的记录整条丢掉（不进重放前缀），而「丢掉」与「判损坏断链」在条款里分不出来"),
+    ("G25", "journal记录头2026-09-14加的fsid8与MAC16，条款只说fsid「与单元头同口径」（系统配置fsid的低8字节）、MAC第一版全0；**读者拿fsid做什么没写**——装置按I-1.4把fsid不符的记录整条丢掉（不进重放前缀），而「丢掉」与「判损坏断链」在条款里分不出来"),
 ];
 
 /// 整条路（mkfs → 取号 → 暖机两次 → 第一个事务）跑一遍，返回落盘后的整份镜像与这次发布的产出。
@@ -3753,8 +3753,8 @@ fn main() {
         ("mapping_key", 27, MAPPING_KEY_BYTES),
         ("mapping_entry", 55, MAPPING_ENTRY_BYTES),
         ("location_entry", 14, LOC_ENTRY),
-        ("superblock", 481, SYSTEM_CONFIGURATION_BYTES),
-        ("superblock_slot", 4096, SYSTEM_CONFIGURATION_SLOT_BYTES),
+        ("system_configuration", 481, SYSTEM_CONFIGURATION_BYTES),
+        ("system_configuration_slot", 4096, SYSTEM_CONFIGURATION_SLOT_BYTES),
     ];
     let mut width_mismatches = 0u64;
     for (structure, expected, actual) in width_rows {
@@ -3902,7 +3902,7 @@ fn main() {
         region_geometry.push(("journal_record", u32::try_from(device_index).expect("设备数"), catalog.journal_offset, catalog.journal_length));
     }
     for device_index in 0..parameters.device_count {
-        region_geometry.push(("superblock", u32::try_from(device_index).expect("设备数"), system_configuration_offset, SYSTEM_CONFIGURATION_SLOT_BYTES));
+        region_geometry.push(("system_configuration", u32::try_from(device_index).expect("设备数"), system_configuration_offset, SYSTEM_CONFIGURATION_SLOT_BYTES));
     }
 
     let impl_snapshot_path = std::env::args().nth(1);
@@ -4075,7 +4075,7 @@ fn main() {
         output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(0)).count()
     ));
     emit(&mut emitter, &format!(
-        "name=instances mkfs_instance={MKFS_INSTANCE_GENERATION} first_writable_mount_instance={FIRST_INSTANCE_GENERATION} mkfs_superblock_generation={SYSTEM_CONFIGURATION_GENERATION_AT_MKFS} acquisition_superblock_generation={SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION} transaction_superblock_generation={} transaction_superblock_slot={}",
+        "name=instances mkfs_instance={MKFS_INSTANCE_GENERATION} first_writable_mount_instance={FIRST_INSTANCE_GENERATION} mkfs_system_configuration_generation={SYSTEM_CONFIGURATION_GENERATION_AT_MKFS} acquisition_system_configuration_generation={SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION} transaction_system_configuration_generation={} transaction_system_configuration_slot={}",
         system_configuration_write_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG)).0,
         system_configuration_write_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG)).1
     ));
@@ -4194,8 +4194,8 @@ mod tests {
         assert_eq!(SYSTEM_CONFIGURATION_CHECKSUM_OFFSET, 155);
         assert_eq!(SYSTEM_CONFIGURATION_FSID_OFFSET, 102);
         assert_eq!(SYSTEM_CONFIGURATION_BYTES, 481);
-        assert_eq!(SYSTEM_CONFIGURATION_SLOT_BYTES, 4096, "超级块槽宽是格式常量（D22 已定项 2，2026-09-14 三方论证后按主 agent 推荐值）");
-        assert_eq!(SYSTEM_CONFIGURATION_SLOT_BYTES - SYSTEM_CONFIGURATION_BYTES, 3615, "481 的超级块在 4096 槽里余 3615");
+        assert_eq!(SYSTEM_CONFIGURATION_SLOT_BYTES, 4096, "系统配置槽宽是格式常量（D22 已定项 2，2026-09-14 三方论证后按主 agent 推荐值）");
+        assert_eq!(SYSTEM_CONFIGURATION_SLOT_BYTES - SYSTEM_CONFIGURATION_BYTES, 3615, "481 的系统配置在 4096 槽里余 3615");
         assert_eq!(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1] - SYSTEM_CONFIGURATION_SLOT_OFFSETS[0], SYSTEM_CONFIGURATION_SLOT_BYTES, "槽距与槽宽同值 ⇒ 两个槽首尾相接、不重叠");
         assert_eq!(SYSTEM_CONFIGURATION_TAIL_OFFSET as u64 + 8 + 4, SYSTEM_CONFIGURATION_BYTES);
         assert_eq!(MAPPING_KEY_BYTES, 27);
@@ -4303,7 +4303,7 @@ mod tests {
     fn mkfs_seeds_three_generation_zero_roots_readable_from_all_regions() {
         let parameters = PoolParameters::settled_two_devices();
         let (recording, genesis) = mkfs(&parameters);
-        let system_configuration = choose_system_configuration(&recording.pool).expect("超级块");
+        let system_configuration = choose_system_configuration(&recording.pool).expect("系统配置");
         let mut readable = 0;
         for region in 0..RING_REGIONS {
             let bytes = recording.pool.read(system_configuration.region_devices[region as usize], ring_slot_offset(region, 0), 512);
@@ -4322,20 +4322,20 @@ mod tests {
         for device_index in 0..2u32 {
             for slot_offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
                 let slot = recording.pool.read(DeviceIdentity(device_index), DeviceOffset(slot_offset), SYSTEM_CONFIGURATION_SLOT_BYTES as usize);
-                let system_configuration = SystemConfiguration::parse_slot(&slot).expect("mkfs 的超级块槽");
+                let system_configuration = SystemConfiguration::parse_slot(&slot).expect("mkfs 的系统配置槽");
                 assert_eq!(system_configuration.slot_generation, 1);
                 assert_eq!(system_configuration.journal_instance, InstanceGeneration(0));
             }
         }
     }
 
-    /// 超级块 481（D22 已定项 9 + 已定项 15，2026-09-14 用户定案加四个字段）：每个字段按**绝对偏移**读一遍，
+    /// 系统配置 481（D22 已定项 9 + 已定项 15，2026-09-14 用户定案加四个字段）：每个字段按**绝对偏移**读一遍，
     /// 以及「整槽校验和罩整个 4096 槽」——改 481 之后那 3615 字节补齐里的任何一个字节，解析都要拒绝。
     #[test]
     fn system_configuration_is_481_bytes_and_the_slot_checksum_covers_all_4096() {
         let BuiltPool { recording, .. } = built_pool();
         let slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1]), SYSTEM_CONFIGURATION_SLOT_BYTES as usize);
-        assert_eq!(slot.len(), 4096, "超级块槽宽是格式常量 4096（D22 已定项 2，2026-09-14 三方论证后）");
+        assert_eq!(slot.len(), 4096, "系统配置槽宽是格式常量 4096（D22 已定项 2，2026-09-14 三方论证后）");
         assert!(SystemConfiguration::parse_slot(&slot).is_some());
         let read_u64 = |offset: usize| u64::from_le_bytes(slot[offset..offset + 8].try_into().expect("切了 8 字节"));
         let read_u32 = |offset: usize| u32::from_le_bytes(slot[offset..offset + 4].try_into().expect("切了 4 字节"));
@@ -4417,9 +4417,9 @@ mod tests {
     }
 
     #[test]
-    /// 取号 [2 个超级块槽]（3 个状态）、暖机第一次 [2 条空记录][根 FUA][2 个超级块槽]（7 个）、第二次 [2][1]（4 个）；
-    /// 第二次的超级块槽写与事务的 16 个单元写之间没有屏障、同一段 18 个（2¹⁸ − 1）；
-    /// 再 [2 条记录][根 FUA][2 个超级块槽]（7 个）⇒ 1 + 3 + 7 + 4 + 262143 + 7 = 262165。
+    /// 取号 [2 个系统配置槽]（3 个状态）、暖机第一次 [2 条空记录][根 FUA][2 个系统配置槽]（7 个）、第二次 [2][1]（4 个）；
+    /// 第二次的系统配置槽写与事务的 16 个单元写之间没有屏障、同一段 18 个（2¹⁸ − 1）；
+    /// 再 [2 条记录][根 FUA][2 个系统配置槽]（7 个）⇒ 1 + 3 + 7 + 4 + 262143 + 7 = 262165。
     fn layer0_state_count_is_262165_with_zero_violations() {
         let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
         let (base, _) = mkfs(&PoolParameters::settled_two_devices());
@@ -4429,7 +4429,7 @@ mod tests {
         let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file());
         assert_eq!(tally.states, 262165);
         assert_eq!(tally.violations, 0, "{:?}", tally.first_violation);
-        assert_eq!(tally.root_persisted_states, 4, "事务根槽持久的状态照旧 4 个：根槽那一段与之后超级块段的子集");
+        assert_eq!(tally.root_persisted_states, 4, "事务根槽持久的状态照旧 4 个：根槽那一段与之后系统配置段的子集");
         // 施加记录会重建那次发布的根（D23 已定项 15）⇒ 事务记录两份都持久、8 个单元都验得过的那 3 个状态也读得到文件。
         assert_eq!(tally.file_read_states, 7);
         assert_eq!(tally.no_file_states, 262158);
@@ -4455,7 +4455,7 @@ mod tests {
     ///
     /// D17（实现分层与第三方管道） 已定项 2 的结构等价类要的是「段边界位置 + 每段步骤种类集合」，
     /// 所以这里连每段的步骤种类多重集一起钉死，四条路径各钉一个**绝对值**（不是拿几条路径互相比）：
-    /// 把某一步录成别的种类——例如超级块槽写录成单元写——段边界与状态数一个都不变，只有这几行会红（C316 ②）。
+    /// 把某一步录成别的种类——例如系统配置槽写录成单元写——段边界与状态数一个都不变，只有这几行会红（C316 ②）。
     #[test]
     fn registered_segment_sequences_match_every_recorded_path() {
         let BuiltPool { recording, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count, .. } = built_pool();
@@ -4466,10 +4466,10 @@ mod tests {
         let warm_up_operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
         let transaction_operations = &recording.operations[warm_up_operation_count..];
         let post_mkfs_operations = &recording.operations[mkfs_operation_count..];
-        assert_eq!(mkfs_operations.len(), 13, "mkfs：11 次写（m1/m2 各两盘、三个第 0 代根、两盘各两个超级块槽）+ 2 道屏障");
+        assert_eq!(mkfs_operations.len(), 13, "mkfs：11 次写（m1/m2 各两盘、三个第 0 代根、两盘各两个系统配置槽）+ 2 道屏障");
         assert_eq!(sizes(mkfs_operations), vec![4, 1, 1, 1, 4]);
         assert_eq!(closed_form_state_count(&split_into_segments(mkfs_operations, true).1), 34);
-        assert_eq!(acquisition_operations.len(), 2, "取号：两盘各写一次超级块槽，不另加屏障");
+        assert_eq!(acquisition_operations.len(), 2, "取号：两盘各写一次系统配置槽，不另加屏障");
         assert_eq!(sizes(acquisition_operations), vec![2]);
         assert_eq!(sizes(warm_up_operations), vec![2, 1, 2, 2, 1, 2]);
         assert_eq!(sizes(transaction_operations), vec![16, 2, 1, 2]);
@@ -4477,24 +4477,24 @@ mod tests {
 
         assert_eq!(
             kinds(mkfs_operations),
-            "[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[superblock_slot×4,barrier]",
-            "mkfs：m1/m2 两个单元各两盘一段、三个第 0 代根各自 FUA 一段、两盘各两个超级块槽收尾"
+            "[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier]",
+            "mkfs：m1/m2 两个单元各两盘一段、三个第 0 代根各自 FUA 一段、两盘各两个系统配置槽收尾"
         );
-        assert_eq!(kinds(acquisition_operations), "[superblock_slot×2]", "取号那一段只有两次超级块槽写");
+        assert_eq!(kinds(acquisition_operations), "[system_configuration_slot×2]", "取号那一段只有两次系统配置槽写");
         assert_eq!(
             kinds(warm_up_operations),
-            "[journal_record×2,barrier×2]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]",
-            "暖机两次空发布：每次「空记录两盘 → 根 FUA → 超级块槽两盘」，第一段前面还有那道开场屏障"
+            "[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]",
+            "暖机两次空发布：每次「空记录两盘 → 根 FUA → 系统配置槽两盘」，第一段前面还有那道开场屏障"
         );
         assert_eq!(
             kinds(transaction_operations),
-            "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]",
-            "第一个事务：8 个单元各两盘 → journal 记录两盘 → 根 FUA → 超级块槽两盘"
+            "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]",
+            "第一个事务：8 个单元各两盘 → journal 记录两盘 → 根 FUA → 系统配置槽两盘"
         );
         assert_eq!(
             kinds(post_mkfs_operations),
-            "[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]",
-            "整条流：取号那两次超级块槽写自成一段（收段的是暖机第一次开头那道屏障），暖机第二次的两个超级块槽与事务的 16 个单元写同一段 18 个写"
+            "[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]",
+            "整条流：取号那两次系统配置槽写自成一段（收段的是暖机第一次开头那道屏障），暖机第二次的两个系统配置槽与事务的 16 个单元写同一段 18 个写"
         );
 
         // 每一步都恰好落在一个段里：各段的步骤数加起来等于录到的操作数。
@@ -4503,7 +4503,7 @@ mod tests {
         }
         // 种类的字母表就这五个，别处不许冒出第六个。
         let alphabet: std::collections::BTreeSet<&str> = segment_step_kinds(post_mkfs_operations, true).concat().iter().map(|kind| kind.tag()).collect();
-        assert_eq!(alphabet.into_iter().collect::<Vec<_>>(), vec!["barrier", "journal_record", "root_record_fua", "superblock_slot", "unit_write"]);
+        assert_eq!(alphabet.into_iter().collect::<Vec<_>>(), vec!["barrier", "journal_record", "root_record_fua", "system_configuration_slot", "unit_write"]);
     }
 
     /// 暖机（D16 已定项 8）：两次空发布，根落区域 1 与区域 2（分住两块盘），jsn 1、2 不点名任何单元，第一个事务从 txg 3 起。
@@ -4513,7 +4513,7 @@ mod tests {
         let operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
         let writes = operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
         let fua_writes: Vec<&WriteRequest> = operations.iter().filter_map(|operation| match operation { RecordedOperation::Write(write) if write.is_fua() => Some(write), RecordedOperation::Write(_) | RecordedOperation::Barrier => None }).collect();
-        assert_eq!(writes, 10, "每次空发布 2 条记录 + 1 个根 + 2 个超级块槽");
+        assert_eq!(writes, 10, "每次空发布 2 条记录 + 1 个根 + 2 个系统配置槽");
         assert_eq!(operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier)).count(), 4);
         assert_eq!(fua_writes.len(), 2);
         let parameters = PoolParameters::settled_two_devices();
@@ -4612,7 +4612,7 @@ mod tests {
         assert_eq!(by_name["data_payload_both_copies"].mapping_fallbacks, 1);
         assert!(matches!(by_name["data_header_last_byte_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
         assert!(matches!(by_name["tree_table_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
-        assert!(matches!(by_name["superblock_slot_one_both_devices"].outcome, RecoveryOutcome::FileRead { .. }));
+        assert!(matches!(by_name["system_configuration_slot_one_both_devices"].outcome, RecoveryOutcome::FileRead { .. }));
     }
 
     /// 根记录 371（D22 已定项 7 的字段表，2026-09-14 用户定案在中央映射树根指针之后加算法类型 1 + nonce 12 + MAC 16）：
```

### `research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs`（原始 diff 第 1047-1059 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs b/research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs
index 2623613..3f71eb6 100644
--- a/research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs
+++ b/research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs
@@ -25,7 +25,7 @@ const NODE_BYTES: u64 = 16384;
 const DIRTY_THRESHOLD_BYTES: u64 = 2 << 30;
 /// 字节表零的预想环长。
 const RING_BYTES: u64 = 64 << 20;
-/// I-8.1（环几何够大） 的安全系数，超级块预想 3。
+/// I-8.1（环几何够大） 的安全系数，系统配置预想 3。
 const SAFETY_FACTOR: u64 = 3;
 /// E44（序号位宽的本机实测与代价）。
 const FSYNC_PER_SECOND: u64 = 2785;
```

### `research/e7-index-bench/src/bin/e147_system_configuration_recompute_from_layout.rs`（原始 diff 第 1060-1090 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e147_system_configuration_recompute_from_layout.rs b/research/e7-index-bench/src/bin/e147_system_configuration_recompute_from_layout.rs
index ca5dc8d..2bd51d9 100644
--- a/research/e7-index-bench/src/bin/e147_system_configuration_recompute_from_layout.rs
+++ b/research/e7-index-bench/src/bin/e147_system_configuration_recompute_from_layout.rs
@@ -1,6 +1,6 @@
-//! E147：超级块字段表按第一个事务字节表重算——D22（单元原子性怎么合成） 已定项 9 挡路条 ③ 那两句「四格数字不可引用」之后的可引用数。
+//! E147：系统配置字段表按第一个事务字节表重算——D22（单元原子性怎么合成） 已定项 9 挡路条 ③ 那两句「四格数字不可引用」之后的可引用数。
 //!
-//! layout/01-first-txn.md 第一节的超级块预想字段表逐行抄进 `SYSTEM_CONFIGURATION_ROWS`，算 512 字节槽的余量与撕裂态，
+//! layout/01-first-txn.md 第一节的系统配置预想字段表逐行抄进 `SYSTEM_CONFIGURATION_ROWS`，算 512 字节槽的余量与撕裂态，
 //! 再给挡路条 ② 还没定的三格各算一个变体。只报数不判输赢；判据与失败条款在 `research/prompts/e147-preregistration.md`。
 
 use e7_index_bench::Emitter;
@@ -35,7 +35,7 @@ impl Segment {
 
 const SEGMENTS: [Segment; 5] = [Segment::Bootstrap, Segment::EncryptionReserve, Segment::Geometry, Segment::Tunable, Segment::Runtime];
 
-/// layout/01-first-txn.md「超级块预想字段表」逐行：段、字段、宽。
+/// layout/01-first-txn.md「系统配置预想字段表」逐行：段、字段、宽。
 const SYSTEM_CONFIGURATION_ROWS: [(Segment, &str, u64); 39] = [
     (Segment::Bootstrap, "magic", 4),
     (Segment::Bootstrap, "format_version", 2),
@@ -45,7 +45,7 @@ const SYSTEM_CONFIGURATION_ROWS: [(Segment, &str, u64); 39] = [
     (Segment::Bootstrap, "device_count", 4),
     (Segment::Bootstrap, "slot_generation", 8),
     (Segment::Bootstrap, "slot_checksum", 32),
-    (Segment::EncryptionReserve, "superblock_mac", 16),
+    (Segment::EncryptionReserve, "system_configuration_mac", 16),
     (Segment::EncryptionReserve, "nonce_watermark", 12),
     (Segment::EncryptionReserve, "kdf_identifier", 2),
     (Segment::EncryptionReserve, "encryption_kind", 1),
```

### `research/e7-index-bench/src/bin/e155_fsync_write_volume.rs`（原始 diff 第 1091-1148 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e155_fsync_write_volume.rs b/research/e7-index-bench/src/bin/e155_fsync_write_volume.rs
index 835fc7e..0f135cb 100644
--- a/research/e7-index-bench/src/bin/e155_fsync_write_volume.rs
+++ b/research/e7-index-bench/src/bin/e155_fsync_write_volume.rs
@@ -20,7 +20,7 @@ const NODE_BYTES: u64 = 16384;
 const UNIT_BYTES: u64 = 32768;
 /// journal 记录固定宽度。
 const RECORD_BYTES: u64 = 4096;
-/// 超级块槽宽度。
+/// 系统配置槽宽度。
 const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
 /// 记录头宽度（`journal_named_items_per_record` 用它反推容量）。
 const RECORD_HEADER_BYTES: u64 = 307;
@@ -971,7 +971,7 @@ mod jia_anchor_tests {
         }
     }
 
-    /// B3（值断言，Q1.6 门槛已撤，第十二节修订②）：G23.1 在锚点的份额，字面口径 47.70%、加超级块 50.07%。
+    /// B3（值断言，Q1.6 门槛已撤，第十二节修订②）：G23.1 在锚点的份额，字面口径 47.70%、加系统配置 50.07%。
     /// 分子（Q1.6）＝ extent、inode 两树的**非叶**节点 + 分配记录、记账、映射、树表四棵的**全部**脏节点 + 根槽；
     /// P=1 时 extent 树高 1（根就是叶），非叶节点为 0——`inode_container_bytes`（叶）与 `extent_bytes`（此格恒为叶）都不算进去。
     #[test]
@@ -984,7 +984,7 @@ mod jia_anchor_tests {
         let share = literal_numerator as f64 / total as f64;
         assert!((share - 0.476_968_796).abs() < 1e-6, "share={share}");
         let with_system_configuration = (literal_numerator + outcome.system_configuration_bytes) as f64 / total as f64;
-        assert!((with_system_configuration - 0.500_742_942).abs() < 1e-6, "with_superblock={with_system_configuration}");
+        assert!((with_system_configuration - 0.500_742_942).abs() < 1e-6, "with_system_configuration={with_system_configuration}");
     }
 
     /// B7（第七节 7.2）：甲，P=145、F1、seq、主几何 —— extent 树长到 2 层，其余树层数与 P=1 相同，
@@ -1326,7 +1326,7 @@ mod write_ahead_log_anchor_tests {
         let jia_fsync = solve_jia_publish(shape_at_file_count_1).total_bytes();
         let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
         let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
-        assert_eq!(jia_fsync - write_ahead_log_full_fsync, 139_776, "甲：四样固定点 × 2 盘 + 根槽 + 超级块槽 × 2");
+        assert_eq!(jia_fsync - write_ahead_log_full_fsync, 139_776, "甲：四样固定点 × 2 盘 + 根槽 + 系统配置槽 × 2");
         assert_eq!(write_ahead_log_full_fsync - write_ahead_log_leaf_fsync, 32_768, "write_ahead_log_full：inode 根 × 2 盘");
 
         let jia_root_slots_in_16 = 16u64; // 甲每次 fsync 都写根槽。
@@ -1658,7 +1658,7 @@ fn one_path_bytes(height: usize, leaf_is_inode_container: bool) -> u64 {
     }
 }
 
-/// Q5.1：甲这一格的 `A(P)`（`total_bytes`）与 `A_tree(P)`（六棵树的字节之和，不含数据单元 / 记录 / 根槽 / 超级块）。
+/// Q5.1：甲这一格的 `A(P)`（`total_bytes`）与 `A_tree(P)`（六棵树的字节之和，不含数据单元 / 记录 / 根槽 / 系统配置）。
 fn total_bytes_and_tree_bytes(shape: PoolShape) -> (u64, u64) {
     let outcome = solve_jia_publish(shape);
     let tree_bytes = outcome.extent_bytes
@@ -1946,7 +1946,7 @@ fn main() {
     println!(
         "{}",
         emitter.emit_raw(&format!(
-            "name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} superblock_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} ring_default_bytes={RING_DEFAULT_BYTES} ring_default_in_flight_limit={} effective_dirty_data_budget_bytes={EFFECTIVE_DIRTY_DATA_BUDGET_BYTES} model=counting file_ops=0",
+            "name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} system_configuration_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} ring_default_bytes={RING_DEFAULT_BYTES} ring_default_in_flight_limit={} effective_dirty_data_budget_bytes={EFFECTIVE_DIRTY_DATA_BUDGET_BYTES} model=counting file_ops=0",
             default_ring_in_flight_limit()
         ))
     );
```

### `research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs`（原始 diff 第 1149-1206 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs b/research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs
index 26f4673..7d95973 100644
--- a/research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs
+++ b/research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs
@@ -37,7 +37,7 @@ const NODE_BYTES: u64 = 16384;
 const UNIT_BYTES: u64 = 32768;
 /// journal 记录固定宽度。
 const RECORD_BYTES: u64 = 4096;
-/// 超级块槽宽度。
+/// 系统配置槽宽度。
 const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
 /// 记录头宽度（`journal_named_items_per_record` 用它反推容量）。
 const RECORD_HEADER_BYTES: u64 = 307;
@@ -1122,7 +1122,7 @@ fn solve_jia_batch_publish(shape: PoolShape, concurrent_fsync_count: u64, shared
         extent_bytes: (extent_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
         inode_container_bytes: (inode_container_dirty * UNIT_BYTES as f64).round() as u64 * DEVICE_COUNT,
         inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
-        // R8 一批只写一份固定点、一条记录组、一个根槽、一份超级块（M9 的反例：每个 fsync 各写一份）。
+        // R8 一批只写一份固定点、一条记录组、一个根槽、一份系统配置（M9 的反例：每个 fsync 各写一份）。
         allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
         accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
         mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
@@ -2200,7 +2200,7 @@ mod write_ahead_log_anchor_tests {
         let jia_fsync = solve_jia_publish(shape_at_file_count_1).total_bytes();
         let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
         let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
-        assert_eq!(jia_fsync - write_ahead_log_full_fsync, 139_776, "甲：四样固定点 × 2 盘 + 根槽 + 超级块槽 × 2");
+        assert_eq!(jia_fsync - write_ahead_log_full_fsync, 139_776, "甲：四样固定点 × 2 盘 + 根槽 + 系统配置槽 × 2");
         assert_eq!(write_ahead_log_full_fsync - write_ahead_log_leaf_fsync, 32_768, "wal_full：inode 根 × 2 盘");
 
         let shape_at_file_count_145 = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
@@ -2422,7 +2422,7 @@ mod row7_group_commit_tests {
     }
 
     /// 阳性对照（5.5，第 7 行）：共享 / 不共享在 B13 那一格上必须给出登记原文的差；
-    /// 甲组提交（concurrent_fsync_count=2 对 concurrent_fsync_count=1）必须省下固定点/记录/根槽/超级块那一份重复开销。
+    /// 甲组提交（concurrent_fsync_count=2 对 concurrent_fsync_count=1）必须省下固定点/记录/根槽/系统配置那一份重复开销。
     #[test]
     fn positive_controls_for_row_7_show_the_prescribed_difference() {
         let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
@@ -2773,7 +2773,7 @@ mod row_2_to_4_counterfactual_tests {
         let share = literal_numerator as f64 / total as f64;
         assert!((share - 0.476_968_796).abs() < 1e-6, "share={share}");
         let with_system_configuration = (literal_numerator + outcome.system_configuration_bytes) as f64 / total as f64;
-        assert!((with_system_configuration - 0.500_742_942).abs() < 1e-6, "with_superblock={with_system_configuration}");
+        assert!((with_system_configuration - 0.500_742_942).abs() < 1e-6, "with_system_configuration={with_system_configuration}");
     }
 }
 
@@ -2843,7 +2843,7 @@ mod row_5_growth_tests {
 
 fn main() {
     let mut emitter = Emitter::new();
-    println!("{}", emitter.emit_raw(&format!("name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} superblock_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} ring_default_bytes={RING_DEFAULT_BYTES} ring_default_in_flight_limit={} model=counting", default_ring_in_flight_limit())));
+    println!("{}", emitter.emit_raw(&format!("name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} system_configuration_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} ring_default_bytes={RING_DEFAULT_BYTES} ring_default_in_flight_limit={} model=counting", default_ring_in_flight_limit())));
     println!(
         "{}",
         emitter.emit_raw(
```

### `research/e7-index-bench/src/bin/e23_journal_geom.rs`（原始 diff 第 1207-1269 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e23_journal_geom.rs b/research/e7-index-bench/src/bin/e23_journal_geom.rs
index 6643545..f8c4f84 100644
--- a/research/e7-index-bench/src/bin/e23_journal_geom.rs
+++ b/research/e7-index-bench/src/bin/e23_journal_geom.rs
@@ -14,7 +14,7 @@
 //! ## 建模的要害：tail 推进与记录写是两件事
 //!
 //! `tail_inline`（XFS 形态）下 tail 只能**搭记录的便车**——不写记录就推不了 tail。
-//! `tail_sb`（jbd2 形态）下 tail 可以在 checkpoint 完成时**单独写一次**推进。
+//! `tail_system_configuration`（jbd2 形态）下 tail 可以在 checkpoint 完成时**单独写一次**推进。
 //! ⇒ 空闲期崩溃时两者的重放量不同，而这正是 jbd2 那次 FUA 买到的东西。
 //! 若把 tail 推进建模成「checkpoint 一完成就免费生效」，两条臂当场相等，实验归零。
 //!
@@ -43,7 +43,7 @@ enum Shape {
 /// 已定项 3 的两条臂。
 #[derive(Clone, Copy, PartialEq, Eq, Debug)]
 enum Tail {
-    /// jbd2 形态：住 journal 超级块，固定位置、原地覆盖、FUA。
+    /// jbd2 形态：住 journal 系统配置，固定位置、原地覆盖、FUA。
     /// 可以脱离记录单独推进 —— 这正是它那次 FUA 买到的东西。
     SystemConfiguration,
     /// XFS 形态：内联在每条记录头的 `tail_lsn` 里。零额外写，但只能搭便车推进。
@@ -53,7 +53,7 @@ enum Tail {
 impl Tail {
     fn label(self) -> &'static str {
         match self {
-            Tail::SystemConfiguration => "tail_sb",
+            Tail::SystemConfiguration => "tail_system_configuration",
             Tail::Inline => "tail_inline",
         }
     }
@@ -241,7 +241,7 @@ mod tests {
         assert!(large_sector_ten_item_record_bytes / small_sector_ten_item_record_bytes < large_sector_one_item_record_bytes / small_sector_one_item_record_bytes, "粒度变粗时 pbs 的影响该变小");
     }
 
-    /// `tail_sb` 省掉头里那 8 字节，必须真的体现在算术里，否则两条臂只是换个名字。
+    /// `tail_system_configuration` 省掉头里那 8 字节，必须真的体现在算术里，否则两条臂只是换个名字。
     #[test]
     fn system_configuration_tail_actually_shrinks_the_header() {
         assert_eq!(Tail::SystemConfiguration.header_bytes(), JOURNAL_RECORD_HEADER_BYTES - 8);
@@ -299,12 +299,12 @@ mod tests {
         let named = vec![1u64; 2000];
         let system_configuration_tail_outcome = simulate_journal_run(&named, Shape::Ring { blocks: 4096 }, Tail::SystemConfiguration, 4096, 1000, Some(1500));
         let inline_tail_outcome = simulate_journal_run(&named, Shape::Ring { blocks: 4096 }, Tail::Inline, 4096, 1000, Some(1500));
-        assert_eq!(system_configuration_tail_outcome.replay_blocks, 0, "超级块 tail 能单独推进，空闲崩溃后不该有重放");
+        assert_eq!(system_configuration_tail_outcome.replay_blocks, 0, "系统配置 tail 能单独推进，空闲崩溃后不该有重放");
         assert_eq!(inline_tail_outcome.replay_blocks, 500, "内联 tail 只能搭便车 ⇒ 崩溃前最后 500 条各占一块");
         assert!(inline_tail_outcome.replay_blocks > system_configuration_tail_outcome.replay_blocks);
     }
 
-    /// **而它买到那个是要付钱的**：超级块 tail 每次推进一次写，内联恒为零。
+    /// **而它买到那个是要付钱的**：系统配置 tail 每次推进一次写，内联恒为零。
     #[test]
     fn system_configuration_tail_costs_exactly_one_write_per_advance() {
         let named = vec![1u64; 5000];
@@ -312,6 +312,6 @@ mod tests {
         let inline_tail_outcome = simulate_journal_run(&named, Shape::Ring { blocks: 4096 }, Tail::Inline, 4096, 1000, None);
         assert_eq!(inline_tail_outcome.tail_blocks, 0, "内联 tail 不该有任何额外写");
         assert_eq!(system_configuration_tail_outcome.checkpoint_count, 5, "5000 次操作、每 1000 次一个 checkpoint");
-        assert_eq!(system_configuration_tail_outcome.tail_blocks, system_configuration_tail_outcome.checkpoint_count, "超级块 tail 的写次数应恰好等于 checkpoint 次数");
+        assert_eq!(system_configuration_tail_outcome.tail_blocks, system_configuration_tail_outcome.checkpoint_count, "系统配置 tail 的写次数应恰好等于 checkpoint 次数");
     }
 }
```

### `research/e7-index-bench/src/bin/e32_journal_timeline.rs`（原始 diff 第 1270-1295 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e32_journal_timeline.rs b/research/e7-index-bench/src/bin/e32_journal_timeline.rs
index e627d93..1d0e8f5 100644
--- a/research/e7-index-bench/src/bin/e32_journal_timeline.rs
+++ b/research/e7-index-bench/src/bin/e32_journal_timeline.rs
@@ -232,7 +232,7 @@ fn setup(slots: usize, settled: u64, stale: u64) -> (Ring, u64, u64, u64) {
         let record = JournalRecord { jsn, timeline: TIMELINE_PREVIOUS_LAP, previous_record_hash: 0, is_checksum_valid: true };
         ring.put(record);
     }
-    let tail = slots as u64 + 1; // 本圈第一条；tail 的权威副本住超级块槽（D23 已定项 3 已定）
+    let tail = slots as u64 + 1; // 本圈第一条；tail 的权威副本住系统配置槽（D23 已定项 3 已定）
 
     // ── 时间线 A：安定段 ──
     let mut chain_hash = anchor;
@@ -270,10 +270,10 @@ fn run(arm: Arm, slots: usize, settled: u64, stale: u64, new_after: u64) -> ArmO
     let resume_jsn = match arm {
         // 没有 `_ =>`
         Arm::ResumeAtPrefix | Arm::BackChain => first_recovery_records.last().map(|record| record.jsn + 1).unwrap_or(tail),
-        // 需要一个「全环见过的最大合法 jsn」水位——本仓的记录头与超级块里都没有这个字段
+        // 需要一个「全环见过的最大合法 jsn」水位——本仓的记录头与系统配置里都没有这个字段
         Arm::SkipHole => {
             // 扫全环取「见过的最大合法 jsn」。⚠️ 这需要一个水位概念，
-            // 而 E23（journal 几何）逐字列出的 11 个记录头字段与超级块槽里都没有它。
+            // 而 E23（journal 几何）逐字列出的 11 个记录头字段与系统配置槽里都没有它。
             let mut highest_valid_jsn = tail;
             for slot in ring.slot.iter().flatten() {
                 if slot.is_checksum_valid && slot.jsn > highest_valid_jsn {
```

### `research/e7-index-bench/src/bin/e37_log_epoch.rs`（原始 diff 第 1296-1310 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e37_log_epoch.rs b/research/e7-index-bench/src/bin/e37_log_epoch.rs
index d17096e..44b9524 100644
--- a/research/e7-index-bench/src/bin/e37_log_epoch.rs
+++ b/research/e7-index-bench/src/bin/e37_log_epoch.rs
@@ -18,8 +18,8 @@
 //!
 //! **epoch 必须持久，而且必须防回滚。** 它住哪儿？
 //! - 住 journal 记录里：恢复要先读记录才知道 epoch，而读记录正需要 epoch 来判——循环。
-//! - 住超级块：那就与 D9（加密）已定项 8 定案里的 **nonce 水位**同类——
-//!   D9（加密）逐字要求水位「不许住在裸明文超级块里……必须落在一个被 MAC 覆盖的字段上」，
+//! - 住系统配置：那就与 D9（加密）已定项 8 定案里的 **nonce 水位**同类——
+//!   D9（加密）逐字要求水位「不许住在裸明文系统配置里……必须落在一个被 MAC 覆盖的字段上」，
 //!   理由是回滚它等于绕过密钥。**epoch 被回滚的后果与之同型**：
 //!   回滚到旧 epoch ⇒ 残留记录重新变成「同一实例」⇒ 出路失效。
 //!
```

### `research/e7-index-bench/src/bin/e43_extension_point_budget.rs`（原始 diff 第 1311-1332 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e43_extension_point_budget.rs b/research/e7-index-bench/src/bin/e43_extension_point_budget.rs
index f41b742..e4f781b 100644
--- a/research/e7-index-bench/src/bin/e43_extension_point_budget.rs
+++ b/research/e7-index-bench/src/bin/e43_extension_point_budget.rs
@@ -2,7 +2,7 @@
 //!
 //! ## 它答什么
 //!
-//! D21 已定「扩展点大小按线在超级块里声明，上限取固定字节数」，
+//! D21 已定「扩展点大小按线在系统配置里声明，上限取固定字节数」，
 //! 但**上限取多少未定案**——正文那个 128 是举例。
 //!
 //! 本实验不问「128 好不好」（那样只可能产出支持性证据），而是**三段夹一个区间**：
@@ -188,7 +188,7 @@ fn slots_per_atomic(slot: u64, atomic: u64) -> u64 {
 
 /// 挂载时求值一次的几何判定。**返回「挂得上 / 挂不上」，不返回「慢一点」**——
 /// 这是 `.claude/rules/fs-design.md`「能不能把『用错了』变成『挂不上』」在本项上的形态。
-/// 输入全部是**操作期间不会变的量**：超级块里声明的 N、格式里的单元与节点大小、
+/// 输入全部是**操作期间不会变的量**：系统配置里声明的 N、格式里的单元与节点大小、
 /// 运行时探测到的 `physical_block_size`。⇒ 挂载时算一次，运行时只查不算。
 fn mount_verdict(extension_point_bytes: u64, unit: u64, node: u64, slot: u64, atomic: u64) -> &'static str {
     if extension_point_bytes > maximum_extension_point_bytes_keeping_payload_positive(unit) {
```

### `research/e7-index-bench/src/bin/e62_ring_home.rs`（原始 diff 第 1333-1365 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e62_ring_home.rs b/research/e7-index-bench/src/bin/e62_ring_home.rs
index 69aff26..94cc887 100644
--- a/research/e7-index-bench/src/bin/e62_ring_home.rs
+++ b/research/e7-index-bench/src/bin/e62_ring_home.rs
@@ -14,7 +14,7 @@
 //! 1. **安全**：设备集合变化序列跑完，「归属指到没有那份数据的盘」次数——
 //!    存身份臂必须恒 0；公式臂 > 0 才说明这个量测得出来。
 //! 2. **安全**：「两个区域归属同一块盘」次数——存身份臂只可能来自 mkfs 那一刻的鸽笼。
-//! 3. **性能**：定位根环的额外读次数。身份与超级块同址 ⇒ 额外读 0；>0 如实记。
+//! 3. **性能**：定位根环的额外读次数。身份与系统配置同址 ⇒ 额外读 0；>0 如实记。
 //! 4. **代价**：字节代价 = R × 设备身份宽度，钉绝对值。
 //!
 //! ## 失败条款（跑前写死）
@@ -27,8 +27,8 @@
 //!
 //! ## 它答不了的
 //!
-//! 计数模型，不是实现：没有超级块、没有真盘、没有挂载。「额外读次数」是按
-//! 「身份存在超级块里、超级块本来就要读」这个前提数出来的，不是实测 I/O。
+//! 计数模型，不是实现：没有系统配置、没有真盘、没有挂载。「额外读次数」是按
+//! 「身份存在系统配置里、系统配置本来就要读」这个前提数出来的，不是实测 I/O。
 
 use e7_index_bench::Emitter;
 
@@ -79,7 +79,7 @@ fn run(arm: Arm, device_count_at_mkfs: u64, events: &[u64]) -> ArmOutcome {
                 }
             }
         }
-        // 身份与超级块同址 ⇒ 定位根环不多读一次；公式臂同样不多读（它算一下就行）
+        // 身份与系统配置同址 ⇒ 定位根环不多读一次；公式臂同样不多读（它算一下就行）
         outcome.extra_reads += 0;
     }
     outcome
```

### `research/e7-index-bench/src/bin/e67_device_subset.rs`（原始 diff 第 1366-1378 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e67_device_subset.rs b/research/e7-index-bench/src/bin/e67_device_subset.rs
index 0f264dd..a019d99 100644
--- a/research/e7-index-bench/src/bin/e67_device_subset.rs
+++ b/research/e7-index-bench/src/bin/e67_device_subset.rs
@@ -2,7 +2,7 @@
 //!
 //! ## 被引用条款逐字贴在这里
 //!
-//! - D2 已定项 6（2026-08-31）：`w = clamp(攒批后写入量 + 1, 2, 4)`，上界 4 是超级块声明的常量。
+//! - D2 已定项 6（2026-08-31）：`w = clamp(攒批后写入量 + 1, 2, 4)`，上界 4 是系统配置声明的常量。
 //! - D2 已定项 6 正文：取消上界时「每条带命中率 1.000 ⇒ 没有一个文件是完整的」——
 //!   **失败模式从「部分受损」变成弥散**，这是本实验要量的那一维。
 //! - D2 已定项 8：定了用几列，没定选哪几块；`.claude/rules/fs-design.md` 硬要求 2
```

### `research/e7-index-bench/src/bin/e71_accounting_keys.rs`（原始 diff 第 1379-1391 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e71_accounting_keys.rs b/research/e7-index-bench/src/bin/e71_accounting_keys.rs
index 3547f8a..22dcd03 100644
--- a/research/e7-index-bench/src/bin/e71_accounting_keys.rs
+++ b/research/e7-index-bench/src/bin/e71_accounting_keys.rs
@@ -12,7 +12,7 @@
 //!   以及：「⚠️ **E71 的公式现在算得出绝对值了**：`统计量数 × 树数 × 设备数 × K`，统计量数 = 9。」
 //! - D5 已定项 2：`K = 根环槽总数 + 1`（保守口径）；每 checkpoint 的记账写次数是
 //!   「`统计量数` 变成 `统计量数 × 2`（一写一删）」；E53 真设备实测的最坏回退是 1 ⇒ K 下界 2。
-//! - D22 已定项 2：根环 **R = 3** 个区域，每区槽数 **S 住超级块，1..16 之间取值不碰任何一条边**
+//! - D22 已定项 2：根环 **R = 3** 个区域，每区槽数 **S 住系统配置，1..16 之间取值不碰任何一条边**
 //!   ⇒ 槽总数 = 3S，保守 `K = 3S + 1`。
 //! - D16 已定项 5：**`T_dirty` = 2 GiB**（2026-08-31 用户定案，明说是初值）。
 //! - E54：「代取 checkpoint 号 + 增量丢弃 ≡ 只留最近 K 代」，且当时的公式
```

### `research/e7-index-bench/src/bin/e75_record_size.rs`（原始 diff 第 1392-1404 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e75_record_size.rs b/research/e7-index-bench/src/bin/e75_record_size.rs
index 5e7c624..005c12a 100644
--- a/research/e7-index-bench/src/bin/e75_record_size.rs
+++ b/research/e7-index-bench/src/bin/e75_record_size.rs
@@ -2,7 +2,7 @@
 //!
 //! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
 //!
-//! - D23 已定项 12（2026-09-01 用户定案）：「定为 **4 KiB，写进超级块**」，
+//! - D23 已定项 12（2026-09-01 用户定案）：「定为 **4 KiB，写进系统配置**」，
 //!   依据逐字是「E45 按『塞满一个定长记录』算出 4096 字节的记录装 `(4096 − 99) / 56 = 71` 项，
 //!   而 D25 已定的目标负载是一次 fsync 带 8 叶、落在 1 条共享脊柱上，E45 记作 12 项事务
 //!   ⇒ **5.9 倍余量**」。并自陈「**那是算术，不是实测**」。
```

### `research/e7-index-bench/src/bin/e79_root_record.rs`（原始 diff 第 1405-1417 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e79_root_record.rs b/research/e7-index-bench/src/bin/e79_root_record.rs
index 42450e3..3820512 100644
--- a/research/e7-index-bench/src/bin/e79_root_record.rs
+++ b/research/e7-index-bench/src/bin/e79_root_record.rs
@@ -8,7 +8,7 @@
 //!
 //! - D22（单元原子性怎么合成）已定项 2：「槽宽 = 挂载时探测的 `physical_block_size`」——
 //!   本机实测这个值是 **512**（D20（承重面：单元的原子性与自包含）推论三第一问）；
-//! - D15（格式冻结政策）已定项 2：「超级块树表 day-1 做成开放列表」——树的数量不封顶；
+//! - D15（格式冻结政策）已定项 2：「系统配置树表 day-1 做成开放列表」——树的数量不封顶；
 //! - D6（快照实现模型）已定项 1：「每头一棵自己的树」——可写头越多树越多。
 //!
 //! 512 字节的槽 × 不封顶的树表 ⇒ 装不下只是时间问题；**什么时候装不下、
```

### `research/e7-index-bench/src/bin/e87_fixed_placement.rs`（原始 diff 第 1418-1513 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e87_fixed_placement.rs b/research/e7-index-bench/src/bin/e87_fixed_placement.rs
index 70d3d29..f0b802d 100644
--- a/research/e7-index-bench/src/bin/e87_fixed_placement.rs
+++ b/research/e7-index-bench/src/bin/e87_fixed_placement.rs
@@ -1,11 +1,11 @@
-//! E87：固定结构的放置 —— 超级块与 journal 环在第一版的 2 盘池上怎么摆，掉一盘各是什么结局。
+//! E87：固定结构的放置 —— 系统配置与 journal 环在第一版的 2 盘池上怎么摆，掉一盘各是什么结局。
 //!
 //! ## 为什么要有这个实验
 //!
-//! C79（超级块与 journal 环的放置没人定）：超级块（设备表、树表、tail 槽、根环参数、阈值
+//! C79（系统配置与 journal 环的放置没人定）：系统配置（设备表、树表、tail 槽、根环参数、阈值
 //! 全住在它里）自己的副本数与放置、journal 环放哪块盘、掉那块盘还挂不挂得上——零覆盖，
 //! 而 2 盘第一版里它们是仅有的单点候选。根环那半已定（D22（单元原子性怎么合成）已定项 2：
-//! R=3 区域、逐区域存设备身份、跨区轮转），E87（固定结构的放置）把它与超级块 / journal
+//! R=3 区域、逐区域存设备身份、跨区轮转），E87（固定结构的放置）把它与系统配置 / journal
 //! 的放置组合起来穷举单盘失效。
 //!
 //! ⚠️ 一条早已有、但从没对准 journal 的规则：D2（RAID 条带策略）已定项 6 的硬下界
@@ -17,33 +17,33 @@
 //! 2 盘；根环 R=3 区域按 mkfs 轮转指派（区域 0、2 → 盘 0，区域 1 → 盘 1；
 //! 逐区域存身份，D2（RAID 条带策略）已定项 7）；发布 g 落区域 `g mod 3`
 //! （D16（发布语义）已定项 6：逐发布计数）。放置臂 2 × 2：
-//! 超级块 {sb_single：只在盘 0 / sb_per_dev：每盘一份}；
+//! 系统配置 {system_configuration_single：只在盘 0 / system_configuration_per_dev：每盘一份}；
 //! journal 环 {j_single：只在盘 0 / j_mirror：两盘各一份，每条记录写两遍}。
 //! 失效：掉盘 0 / 掉盘 1，逐发布代 g = 1..=12 穷举。
 //!
-//! 判定（每格）：挂得上吗（≥1 超级块副本 且 ≥1 幸存根）；根回退几代（幸存区域里
+//! 判定（每格）：挂得上吗（≥1 系统配置副本 且 ≥1 幸存根）；根回退几代（幸存区域里
 //! 最新的代与 g 的差，取 g 全档最坏）；journal 重放窗口丢不丢（环副本全灭 = 丢，
 //! 上界一个 T_time 窗口，D16（发布语义）已定项 5）；稳态代价（镜像 journal 每条记录 +1 写）。
 //!
 //! ⚠️ **试跑时发现并修订的一处（2026-09-02，任何正式轮入库之前）**：初版没建 mkfs 的
 //! 初始根，g=1 掉盘 1 直接全灭（第一次发布恰落在盘 1）。⇒ 模型改成
 //! **mkfs 把第 0 代根种进全部区域**，并保留未种臂把这个洞钉成单测——
-//! 它是要写进 C79（超级块与 journal 环的放置没人定）收口决策的一条硬要求。
+//! 它是要写进 C79（系统配置与 journal 环的放置没人定）收口决策的一条硬要求。
 //!
 //! ## 判据（跑前写死，跑完不许改）
 //!
-//! 1. sb_single 掉盘 0 必不可挂（哪怕数据、根、journal 全健在）——单点的机检形态；
-//!    sb_per_dev 全部 8 格可挂。
+//! 1. system_configuration_single 掉盘 0 必不可挂（哪怕数据、根、journal 全健在）——单点的机检形态；
+//!    system_configuration_per_dev 全部 8 格可挂。
 //! 2. 根回退最坏值按轮转算术钉死：掉盘 1（只有区域 1）最坏 1 代；
 //!    掉盘 0（区域 0、2 全灭、只剩区域 1）最坏 **2** 代——g ≡ 1 (mod 3) 时区域 1 里
 //!    最新的是 g−3？不对：区域 1 存代 ≡ 1 (mod 3)，g ≡ 1 时它自己就在盘 1……
 //!    穷举给答案，判据只钉「与逐代穷举一致的闭式」（见单测手算）。
 //! 3. j_single 掉盘 0 丢重放窗口，j_mirror 全格不丢；镜像的稳态代价恰为每条记录 ×2。
-//! 4. 不判「选哪格」——那是 C79（超级块与 journal 环的放置没人定）的收口决策，交表。
+//! 4. 不判「选哪格」——那是 C79（系统配置与 journal 环的放置没人定）的收口决策，交表。
 //!
 //! ## 它答不了的
 //!
-//! 纯算术穷举，文件操作 0 处。不建模超级块的更新协议（≥2 槽轮换那套语义归
+//! 纯算术穷举，文件操作 0 处。不建模系统配置的更新协议（≥2 槽轮换那套语义归
 //! D23（journal 的角色与格式）已定项 3 的同型纪律，收口时一起定）；不建模盘失而复得
 //! （那是 C88（根环的时间线判别未实现）的射程）；S（每区槽数）取 1 简化——
 //! 槽多只加深同区回退，不改跨区结局。
@@ -126,7 +126,7 @@ fn judge(system_configuration_arm: SystemConfigurationArm, journal_arm: JournalA
         JournalArm::Mirror => false,
     };
     Cell {
-        mountable: system_configuration_survives, // 根恒有幸存者，成不成只看超级块
+        mountable: system_configuration_survives, // 根恒有幸存者，成不成只看系统配置
         worst_fallback,
         window_lost,
         journal_writes_per_record: match journal_arm {
@@ -151,7 +151,7 @@ fn main() {
                 println!(
                     "{}",
                     emitter.emit_raw(&format!(
-                        "name=cell sb={} journal={} dead_dev={dead_device} mountable={} worst_root_fallback={} replay_window_lost={} journal_writes_per_record={}",
+                        "name=cell system_configuration={} journal={} dead_dev={dead_device} mountable={} worst_root_fallback={} replay_window_lost={} journal_writes_per_record={}",
                         match system_configuration_arm { SystemConfigurationArm::Single => "single", SystemConfigurationArm::PerDevice => "per_dev" },
                         match journal_arm { JournalArm::Single => "single", JournalArm::Mirror => "mirror" },
                         u8::from(cell.mountable),
@@ -170,7 +170,7 @@ fn main() {
 mod tests {
     use super::*;
 
-    /// **判据 1**：超级块单份、掉盘 0 ⇒ 不可挂（数据、根、journal 健在也没用）；
+    /// **判据 1**：系统配置单份、掉盘 0 ⇒ 不可挂（数据、根、journal 健在也没用）；
     /// 每盘一份 ⇒ 全部 8 格可挂。
     #[test]
     fn single_system_configuration_is_a_single_point_of_failure() {
@@ -200,7 +200,7 @@ mod tests {
     }
 
     /// 根恒有幸存者（**前提：mkfs 把第 0 代根种进全部区域**）：任何单盘失效、
-    /// 任何代都找得到根。这是「挂得上只看超级块」那半句的前提。
+    /// 任何代都找得到根。这是「挂得上只看系统配置」那半句的前提。
     #[test]
     fn roots_always_survive_single_disk_loss() {
         for dead_device in 0..DEVICE_COUNT {
```

### `research/e7-index-bench/src/bin/e97_entry_encoding.rs`（原始 diff 第 1514-1526 行）

```diff
diff --git a/research/e7-index-bench/src/bin/e97_entry_encoding.rs b/research/e7-index-bench/src/bin/e97_entry_encoding.rs
index 28933d5..73c5a0e 100644
--- a/research/e7-index-bench/src/bin/e97_entry_encoding.rs
+++ b/research/e7-index-bench/src/bin/e97_entry_encoding.rs
@@ -32,7 +32,7 @@
 //! - **D8 write buffer 硬要求 3 逐字**：「**条目必须自带序号（seq），这是格式要求**」。
 //!   ⚠️ **第三轮修正**：第一、二版的段清单里根本没有 seq，
 //!   而判据 2 只查已列出的段 ⇒ 漏掉一整段是它的盲区。判据 9 就是补这个。
-//! - D22 已定项 2：根环 **R = 3** 区域，每区槽数 **S 住超级块、1..16**
+//! - D22 已定项 2：根环 **R = 3** 区域，每区槽数 **S 住系统配置、1..16**
 //!   ⇒ 槽总数 3S，`K = 3S + 1 ≤ 49`。
 //! - D22 已定项 7：根记录里 `checkpoint_txg` **8 字节**。
 //! - D16 已定项 5：`T_dirty` = **2 GiB**。
```

### `research/mutations/e126_system_configuration_slot_width.tsv`（原始 diff 第 1527-1536 行）

```diff
diff --git a/research/mutations/e126_system_configuration_slot_width.tsv b/research/mutations/e126_system_configuration_slot_width.tsv
index 0d542d4..59e2779 100644
--- a/research/mutations/e126_system_configuration_slot_width.tsv
+++ b/research/mutations/e126_system_configuration_slot_width.tsv
@@ -1,4 +1,4 @@
-# E126 超级块槽宽四条候选的代价 —— 变异表
+# E126 系统配置槽宽四条候选的代价 —— 变异表
 # 每行三段：变异名<TAB>原文<TAB>替换文
 min_unit_bytes	const MINIMUM_UNIT_BYTES: u64 = 16384;	const MINIMUM_UNIT_BYTES: u64 = 4096;
 first_version_disks	const FIRST_VERSION_DISKS: u64 = 2;	const FIRST_VERSION_DISKS: u64 = 3;
```

### `research/mutations/e155_fsync_write_volume.tsv`（原始 diff 第 1537-1556 行）

```diff
diff --git a/research/mutations/e155_fsync_write_volume.tsv b/research/mutations/e155_fsync_write_volume.tsv
index ff5a6cf..c89e24f 100644
--- a/research/mutations/e155_fsync_write_volume.tsv
+++ b/research/mutations/e155_fsync_write_volume.tsv
@@ -1,13 +1,13 @@
 M1_两块盘改成一块	const DEVICE_COUNT: u64 = 2;	const DEVICE_COUNT: u64 = 1;
 M2_甲不写映射树	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: 0,
-M3_甲不写超级块槽	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: 0,
+M3_甲不写系统配置槽	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: 0,
 M4_根槽宽恒4096	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: 4096,
 M5_inode树允许1层	let inode_layers = tree_layers(file_count, inode_leaf_cap, inode_internal_cap, 2);	let inode_layers = tree_layers(file_count, inode_leaf_cap, inode_internal_cap, 1);
 M6_WAL的fsync多写一个根槽宽度的字节	data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT,\n        extent_bytes: (extent_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,	data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT + PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES,\n        extent_bytes: (extent_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
 M7_乙在fsync写全部祖先	let extent_leaf = extent_dirty[0];	let extent_leaf = sum_f64(&extent_dirty);
 M9_摊销分母取N加1	(fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / interval_fsyncs as f64	(fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / (interval_fsyncs as f64 + 1.0)
 M12_散组不饱和直接返回触达数	expected_saturated_node_count * (1.0 - (1.0 - 1.0 / expected_saturated_node_count).powf(touch_count))	touch_count
-M15_G23_1分子加超级块槽	+ self.root_slot_bytes\n    }	+ self.root_slot_bytes\n            + self.system_configuration_bytes\n    }
+M15_G23_1分子加系统配置槽	+ self.root_slot_bytes\n    }	+ self.root_slot_bytes\n            + self.system_configuration_bytes\n    }
 M19_环安全系数改成2	const RING_SAFETY_FACTOR_MAIN: u64 = 3;	const RING_SAFETY_FACTOR_MAIN: u64 = 2;
 M10_一条记录容量改成71	const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;	const NAMED_ITEMS_PER_RECORD_MAIN: u64 = 71;
 M17_Q3_1不计头H	let rounded = ((header as f64 + conditions_per_node * entry_width as f64) / pbs as f64).ceil() * pbs as f64;	let rounded = ((conditions_per_node * entry_width as f64) / pbs as f64).ceil() * pbs as f64;
```

### `research/mutations/e155_second_run_fsync_write_volume.tsv`（原始 diff 第 1557-1585 行）

```diff
diff --git a/research/mutations/e155_second_run_fsync_write_volume.tsv b/research/mutations/e155_second_run_fsync_write_volume.tsv
index ef031d1..585c22f 100644
--- a/research/mutations/e155_second_run_fsync_write_volume.tsv
+++ b/research/mutations/e155_second_run_fsync_write_volume.tsv
@@ -1,13 +1,13 @@
 M1_两块盘改成一块	const DEVICE_COUNT: u64 = 2;	const DEVICE_COUNT: u64 = 1;
 M2_甲不写映射树	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: 0,
-M3_甲不写超级块槽	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: 0,
+M3_甲不写系统配置槽	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: 0,
 M4_根槽宽恒4096	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,	inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: 4096,
 M5_inode树允许1层	    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));\n    let inode_layers = tree_layers(file_count, inode_leaf_cap, inode_internal_cap, 2);	    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));\n    let inode_layers = tree_layers(file_count, inode_leaf_cap, inode_internal_cap, 1);
 M6_WAL的fsync多写一个根槽宽度的字节	data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT,\n        extent_bytes: (extent_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,	data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT + PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES,\n        extent_bytes: (extent_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
 M7_乙在fsync写全部祖先	            (extent_total, inode_container, inode_root, 2u64)\n        }\n        WriteAheadLogArm::WriteAheadLogLeaf => {\n            let extent_leaf = extent_dirty[0];	            (extent_total, inode_container, inode_root, 2u64)\n        }\n        WriteAheadLogArm::WriteAheadLogLeaf => {\n            let extent_leaf = sum_f64(&extent_dirty);
 M9_摊销分母取N加1	(fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / interval_fsyncs as f64	(fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / (interval_fsyncs as f64 + 1.0)
 M12_散组不饱和直接返回触达数	expected_saturated_node_count * (1.0 - (1.0 - 1.0 / expected_saturated_node_count).powf(touch_count))	touch_count
-M15_G23_1分子加超级块槽	+ self.root_slot_bytes\n    }	+ self.root_slot_bytes\n            + self.system_configuration_bytes\n    }
+M15_G23_1分子加系统配置槽	+ self.root_slot_bytes\n    }	+ self.root_slot_bytes\n            + self.system_configuration_bytes\n    }
 M19_环安全系数改成2	const RING_SAFETY_FACTOR_MAIN: u64 = 3;	const RING_SAFETY_FACTOR_MAIN: u64 = 2;
 M10_一条记录容量改成71	const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;	const NAMED_ITEMS_PER_RECORD_MAIN: u64 = 71;
 M17_Q3_1不计头H	let rounded = ((header as f64 + conditions_per_node * entry_width as f64) / pbs as f64).ceil() * pbs as f64;	let rounded = ((conditions_per_node * entry_width as f64) / pbs as f64).ceil() * pbs as f64;
@@ -25,7 +25,7 @@ R2_M14_摊销分母取N加1	(fsync_total_bytes as f64 * interval_fsyncs as f64 +
 R2_M15_乙M在一批里写全部祖先	            (extent_total, inode_container, inode_root, 2u64)\n        }\n        WriteAheadLogArm::WriteAheadLogLeaf => {\n            let extent_leaf = extent_dirty[0];	            (extent_total, inode_container, inode_root, 2u64)\n        }\n        WriteAheadLogArm::WriteAheadLogLeaf => {\n            let extent_leaf = sum_f64(&extent_dirty);
 R2_M18_K9prime中间版不进积压	let release_backlog_units = previous_written_units + distinct_data_units_touched + extra_allocation_release_insertions;	let release_backlog_units = previous_written_units + distinct_data_units_touched;
 R2_M19_甲按WAL臂算中间版结构恒零	        0.0, // 甲没有中间版（K9′ 只对 WAL 两臂有意义）。	        (1.0_f64 * 0.0_f64), // R2_M19：甲按 WAL 臂的中间版公式算，结构上恒 0（每批就是一次发布），预期等价
-R2_M9_甲组提交每个fsync各写一份固定点	        // R8 一批只写一份固定点、一条记录组、一个根槽、一份超级块（M9 的反例：每个 fsync 各写一份）。\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,	        // R8 一批只写一份固定点、一条记录组、一个根槽、一份超级块（M9 的反例：每个 fsync 各写一份）。\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT * concurrent_fsync_count,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT * concurrent_fsync_count,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT * concurrent_fsync_count,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT * concurrent_fsync_count,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT * concurrent_fsync_count,\n        root_slot_bytes: pbs * concurrent_fsync_count,\n        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT * concurrent_fsync_count,
+R2_M9_甲组提交每个fsync各写一份固定点	        // R8 一批只写一份固定点、一条记录组、一个根槽、一份系统配置（M9 的反例：每个 fsync 各写一份）。\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,\n        root_slot_bytes: pbs,\n        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,	        // R8 一批只写一份固定点、一条记录组、一个根槽、一份系统配置（M9 的反例：每个 fsync 各写一份）。\n        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT * concurrent_fsync_count,\n        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT * concurrent_fsync_count,\n        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT * concurrent_fsync_count,\n        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT * concurrent_fsync_count,\n        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT * concurrent_fsync_count,\n        root_slot_bytes: pbs * concurrent_fsync_count,\n        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT * concurrent_fsync_count,
 R2_M10_WAL两臂一批写concurrent_fsync_count条记录	    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);\n    let unit_count = (batch_data_unit_count as f64 + extent_component + inode_container_component + inode_root_component).round() as u64;	    let record_count = concurrent_fsync_count;\n    let unit_count = (batch_data_unit_count as f64 + extent_component + inode_container_component + inode_root_component).round() as u64;
 R2_M11_不共享照共享算	    let extent_touch = if shared {\n        TouchSpecification::Continuous(batch_data_unit_count as f64)\n    } else {\n        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 }\n    };\n    let inode_touch = if shared {\n        TouchSpecification::Continuous(concurrent_fsync_count as f64)\n    } else {\n        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 }\n    };	    let extent_touch = if !shared {\n        TouchSpecification::Continuous(batch_data_unit_count as f64)\n    } else {\n        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 }\n    };\n    let inode_touch = if !shared {\n        TouchSpecification::Continuous(concurrent_fsync_count as f64)\n    } else {\n        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 }\n    };
 R2_M12_共享照不共享算	    let extent_touch = if shared {\n        TouchSpecification::Continuous(batch_data_unit_count as f64)\n    } else {\n        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 }\n    };\n    let inode_touch = if shared {\n        TouchSpecification::Continuous(concurrent_fsync_count as f64)\n    } else {\n        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 }\n    };	    let extent_touch = if !shared {\n        TouchSpecification::Continuous(batch_data_unit_count as f64)\n    } else {\n        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 }\n    };\n    let inode_touch = if !shared {\n        TouchSpecification::Continuous(concurrent_fsync_count as f64)\n    } else {\n        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 }\n    };
```

### `research/mutations/e87_fixed_placement.tsv`（原始 diff 第 1586-1595 行）

```diff
diff --git a/research/mutations/e87_fixed_placement.tsv b/research/mutations/e87_fixed_placement.tsv
index 9d57177..b307185 100644
--- a/research/mutations/e87_fixed_placement.tsv
+++ b/research/mutations/e87_fixed_placement.tsv
@@ -1,4 +1,4 @@
-M1_单份超级块掉盘0也算可挂	SystemConfigurationArm::Single => dead_device != 0,	SystemConfigurationArm::Single => true,
+M1_单份系统配置掉盘0也算可挂	SystemConfigurationArm::Single => dead_device != 0,	SystemConfigurationArm::Single => true,
 M2_种子退化成只有区域0	if candidate_generation == 0 {\n            return generations_back; // 第 0 代在全部区域都有种子 ⇒ 任何盘上都找得到\n        }	if candidate_generation == 0 {\n            return if dead_device == 0 { u64::MAX } else { generations_back };\n        }
 M3_区域指派全落一盘	region % DEVICE_COUNT	0
 M4_镜像journal也丢窗口	JournalArm::Mirror => false,	JournalArm::Mirror => dead_device == 0,
```

### `research/scripts/archive-past-rounds.py`（原始 diff 第 1596-1697 行）

```diff
diff --git a/research/scripts/archive-past-rounds.py b/research/scripts/archive-past-rounds.py
index 93435d9..ed6b610 100755
--- a/research/scripts/archive-past-rounds.py
+++ b/research/scripts/archive-past-rounds.py
@@ -79,6 +79,46 @@ def rewrite_links(root, doomed):
     return changed, touched
 
 
+def gates_losing_input(root, doomed):
+    """删完之后会失去输入的检查：门禁与研究脚本里按名字点到的留存产物，删了那道检查就没得比。
+    判据与排除表那条同形——**被某道检查当输入的东西不能删**。
+    2026-09-21 实测：归档删掉 E142 的入库产物之后，门禁 52 号（段序列登记表与 E142 产物逐字比对）
+    与它的自证一起塌，而归档脚本当时只检查了链接与排除表，一个字都没说。"""
+    import glob as _glob
+    doomed_names = {os.path.basename(p) for p in doomed}
+    losing = []
+    for script in sorted(_glob.glob(os.path.join(root, ".claude", "gate.d", "*.sh"))
+                         + _glob.glob(os.path.join(root, "research", "scripts", "*"))):
+        if os.path.isdir(script) or os.path.basename(script) == os.path.basename(__file__):
+            continue
+        try:
+            text = open(script, encoding="utf-8", errors="ignore").read()
+        except OSError:
+            continue
+        for name in doomed_names:
+            if name in text:
+                losing.append("%s  ← 点名了 %s" % (os.path.relpath(script, root), name))
+                break
+    return losing
+
+
+def stale_exclusions(root):
+    """删完之后指不到东西的排除项：排除表指向不存在的路径时，那道检查自己会判红
+    （`.claude/singlefs-ai-sop/rules/show-me-test.md`「排除的每一份都要登记」）。
+    2026-09-21 实测：归档删掉 research/results/e153-s1-probe-tests/ 之后，naming-lint 整道红。"""
+    import glob as _glob
+    stale = []
+    for table in sorted(_glob.glob(os.path.join(root, ".claude", "*-exclude"))):
+        for number, line in enumerate(open(table, encoding="utf-8"), 1):
+            line = line.strip()
+            if not line or line.startswith("#"):
+                continue
+            target = line.split("#")[0].strip()
+            if target and not os.path.exists(os.path.join(root, target)):
+                stale.append("%s:%d  %s" % (os.path.relpath(table, root), number, target))
+    return stale
+
+
 def run(root, apply_changes):
     doomed = past_round_files(root)
     if not doomed:
@@ -102,7 +142,20 @@ def run(root, apply_changes):
     changed, touched = rewrite_links(root, doomed)
     for relative in doomed:
         os.remove(os.path.join(root, relative))
+    stale = stale_exclusions(root)
+    losing = gates_losing_input(root, doomed)
     print("  ✓ 删了 %d 份上一轮及更早的实验记录；%d 份文件里 %d 处引用去掉了路径、只留文件名" % (len(doomed), touched, changed))
+    if losing:
+        print("  ! 这几处检查点名了刚删掉的产物，它们会失去输入：")
+        for entry in losing:                                       # gate-lint:detail
+            print("      %s" % entry)
+        print("     → 怎么办：被某道检查当输入的产物不该归档——把它从 git 取回（git show <提交>^:<路径>），")
+        print("               或者重跑那个实验产生新产物；两种都做不了就改那道检查，别让它悄悄没得比。")
+    if stale:
+        print("  ! 删完之后这几条排除项指不到东西了，各自那道检查会判红：")
+        for entry in stale:                                        # gate-lint:detail
+            print("      %s" % entry)
+        print("     → 怎么办：把它们从各自的排除表里删掉——排除只缩不涨，指不到的排除会让人以为那批文件已经被绕开了。")
     print("     → 下一步：跑门禁 23 号（文档指向）确认没有指空的链接，再跑一次本脚本 --check 判绿。")
     return 0
 
@@ -140,6 +193,19 @@ def selftest():
             print("  ✗ 自检失败：本轮新产生的被删了")
             print("     → 怎么办：未跟踪的文件不在 ls-files 里，看 past_round_files() 为什么把它算进去了。")
             return 1
+        # 「删完有没有门禁失去输入」这条自己也要证明会红：造一道点名了被删产物的检查
+        gate_dir = os.path.join(work, ".claude", "gate.d")
+        os.makedirs(gate_dir, exist_ok=True)
+        open(os.path.join(gate_dir, "99-sample.sh"), "w").write(
+            '#!/usr/bin/env bash\n# 点名 research/results/e1-old.out 当输入\n')
+        if not gates_losing_input(work, ["research/results/e1-old.out"]):
+            print("  ✗ 自检失败：有检查点名了被删的产物，gates_losing_input 却一条都没报")
+            print("     → 怎么办：看它扫的目录与 doomed_names 的匹配。")
+            return 1
+        if gates_losing_input(work, ["research/results/e2-new.out"]):
+            print("  ✗ 自检失败：没有检查点名 e2-new.out，gates_losing_input 却报了")
+            print("     → 怎么办：它在拿文件名做子串匹配，看是不是打中了别的词。")
+            return 1
         page = open(os.path.join(work, ".claude/kb/page.md"), encoding="utf-8").read()
         if "research/results/e1-old.out" in page:
             print("  ✗ 自检失败：指向被删文件的引用还留着指空的路径")
@@ -153,7 +219,8 @@ def selftest():
             print("  ✗ 自检失败：删完 --check 仍判红")
             print("     → 怎么办：看 past_round_files() 删除之后还认出了什么。")
             return 1
-    print("  ✓ 自检：有旧记录判红、删完判绿、判决与本轮新产物不被删、指向被删文件的引用改成不带路径的说法")
+    print("  ✓ 自检：有旧记录判红、删完判绿、判决与本轮新产物不被删、指向被删文件的引用改成不带路径的说法、"
+          "点名被删产物的检查报得出来且不误报")
     return 0
 
 
```

### `research/scripts/e72-devtable-probe.sh`（原始 diff 第 1698-1728 行）

```diff
diff --git a/research/scripts/e72-devtable-probe.sh b/research/scripts/e72-devtable-probe.sh
index 7991c46..c564cb3 100755
--- a/research/scripts/e72-devtable-probe.sh
+++ b/research/scripts/e72-devtable-probe.sh
@@ -3,7 +3,7 @@
 #
 # ## 被引用条款逐字（verify-before-claiming.md「把定义句原样贴进注释」）
 #
-# - D12 已定项 3：「设备级几何量**住超级块里的设备描述符表**，不住块头。」
+# - D12 已定项 3：「设备级几何量**住系统配置里的设备描述符表**，不住块头。」
 # - D12 已定项 4：「**凡是进入地址算术的几何量都要逐设备比对，不进地址算术的不比。**」
 #   清单四条：`physical_block_size`（对不上 ⇒ 拒绝可写挂载）、
 #   设备容量（变小 ⇒ 拒绝；变大 ⇒ 放行并记）、设备身份（⇒ 拒绝挂载）、
@@ -28,7 +28,7 @@
 #
 # - 设备是**真 loop 设备**，几何从 `/sys/block/*/queue` **实测读出**，不是编的。
 #   「改坏几何」用真手段：换 `--sector-size` 重挂、`truncate` 改容量、换设备顺序。
-# - **挂载路径本身不存在**（无超级块、无实现）⇒ 这里实现的是 D12 已定项 4 那条**规则**，
+# - **挂载路径本身不存在**（无系统配置、无实现）⇒ 这里实现的是 D12 已定项 4 那条**规则**，
 #   验的是**规则里每一条比对项有没有判别力**，不是「我们的挂载代码对不对」。
 # - 表的条目宽度**仓里没定过** ⇒ 是本脚本的假设，按三档各算一次。
 set -uo pipefail
@@ -83,7 +83,7 @@ probe() { # $1=loop 设备
     "$(S blockdev --getsize64 "$1")"
 }
 
-# mkfs：把实测几何写进描述符表文件（模拟超级块那张表）
+# mkfs：把实测几何写进描述符表文件（模拟系统配置那张表）
 write_table() { # $1=表文件
   : > "$1"
   local i g
```

### `research/scripts/fixtures/stale-candidates-benchmark-facts.tsv`（原始 diff 第 1729-1741 行）

```diff
diff --git a/research/scripts/fixtures/stale-candidates-benchmark-facts.tsv b/research/scripts/fixtures/stale-candidates-benchmark-facts.tsv
index 1fba25d..2c94fae 100644
--- a/research/scripts/fixtures/stale-candidates-benchmark-facts.tsv
+++ b/research/scripts/fixtures/stale-candidates-benchmark-facts.tsv
@@ -3,7 +3,7 @@ A1	新立：C374 这条欠账编号	C374（释放代与树表诞生 txg 只有
 A2	新立：C359 到 C364 这六条欠账编号	C359 到 C364 六条新立（挂载判定头宽、journal 算法类型预留、打包容器新类头宽/槽宽/码、保留池树高、整理意图记录字段表）	C359|C360|C361|C362|C363|C364	H2
 A3	C73（分配记录的态别没定） 的 map_provenance 分项号	C73（分配记录的态别没定） 的 map_provenance 分项号从 D18（块里携带什么信息） 已定项 3 改到已定项 5	C73	H2
 A4	C323（镜像大小全仓没有条款） 的行首	C323（镜像大小全仓没有条款） 行首按今天的 4 GiB 与产物读数改	C323	H2
-A5	C245（超级块槽的写频率，两条条款说反话） 的记法	C245（超级块槽的写频率，两条条款说反话） 补记「每个 checkpoint 一次」与「每次发布一次」是同一个频率	C245	H2
+A5	C245（系统配置槽的写频率，两条条款说反话） 的记法	C245（系统配置槽的写频率，两条条款说反话） 补记「每个 checkpoint 一次」与「每次发布一次」是同一个频率	C245	H2
 A6	C327（变异表的锚点腐化没有会红的检查） 状态	C327（变异表的锚点腐化没有会红的检查） 还清：门禁 33 号加锚点子串计数、4 条腐化锚点各跑一次变异	C327	H3
 A7	新立：C350 到 C358 这九条欠账编号	C350 到 C358 九条新立（计时实验负载、ε 作废输入、day-1 定案重开、journal 承重、zoned 三方、两条重开判据、总审核回扫闸等）	C350|C351|C352|C353|C354|C355|C356|C357|C358	H3
 A8	C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 与 C113（扫描重建时多版单元的现行版本判定无输入） 第四列	C104 与 C113 第四列词从「已还」改成「决策已定、检查仍欠」——与已还清表同一个词，检索时分不开	C104|C113	H3
```

### `research/scripts/replay.sh`（原始 diff 第 1742-1785 行）

```diff
diff --git a/research/scripts/replay.sh b/research/scripts/replay.sh
index 2b4144e..3b9cb14 100755
--- a/research/scripts/replay.sh
+++ b/research/scripts/replay.sh
@@ -405,7 +405,7 @@ fi
 
 cargo build --release --manifest-path e7-index-bench/Cargo.toml >/dev/null 2>&1 || { echo "replay: 构建失败" >&2; exit 2; }
 
-pass=0; drift=0; timing_only=0; broken=0; claim_bad=0
+pass=0; drift=0; timing_only=0; broken=0; claim_bad=0; archived=0
 CLAIM_QUEUE=()
 
 printf '%-5s %-24s %-10s %s\n' 实验 二进制 判定 说明
@@ -442,6 +442,13 @@ while IFS='|' read -r exp bin args stored kind; do
   if [[ -n "$gate2" ]]; then
     printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "$gate2"; broken=$((broken+1)); continue
   fi
+  # 留存产物已按「每次提交删上一次的实验记录」归档进版本库时，这一档不比对。
+  # 不报成「对不上」：那与「装置真的改坏了、复跑出不同字节」长得一模一样，读的人会以为实验坏了
+  # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
+  if [[ ! -f "results/$stored" ]]; then
+    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 产物已归档 "$stored 不在树里；本次跑得出来，逐字节这一档不比对"
+    archived=$((archived+1)); CLAIM_QUEUE+=("$exp|$fresh"); continue
+  fi
   if diff -q "$fresh" "results/$stored" >/dev/null 2>&1; then
     printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 字节一致 "$stored"; pass=$((pass+1))
     CLAIM_QUEUE+=("$exp|$fresh"); continue
@@ -466,8 +473,15 @@ for q in "${CLAIM_QUEUE[@]}"; do
   check_claims "${q%%|*}" "${q#*|}" || claim_bad=$((claim_bad+1))
 done
 printf '%s\n' "-------------------------------------------------------------------------"
-echo "字节一致 $pass ／ 仅计时不同 $timing_only ／ 对不上 $drift ／ 跑不了 $broken ／ 结论断言不中 $claim_bad"
+echo "字节一致 $pass ／ 仅计时不同 $timing_only ／ 对不上 $drift ／ 跑不了 $broken ／ 结论断言不中 $claim_bad ／ 产物已归档 $archived"
 echo "本轮输出：$OUT_DIR"
+if [[ $archived -ne 0 ]]; then
+  echo "  ! 「产物已归档」$archived 行：留存产物按「每次提交删上一次的实验记录」归档进了版本库，逐字节这一档没有对照物。"
+  echo "     这不是判红——这几行本次都跑得出来，结论区间断言照常判。要看当时的产物："
+  echo "         git log --all --diff-filter=D --name-only -- \"*<产物文件名>\"     # 找到删它的那次提交"
+  echo "         git show <提交>^:research/results/<产物文件名>                      # 读回当时的内容"
+  echo "     读到的是当时的数、不是今天的结论；要拿它支撑新结论就重新跑一遍（evidence-discipline.md）。"
+fi
 if [[ $drift -ne 0 || $broken -ne 0 || $claim_bad -ne 0 ]]; then
   echo "  → 怎么办：「跑不了」看上面那一行给的 $OUT_DIR/<实验号>.err；「对不上」按上面给的 diff 命令看差在哪，" \
        "结构性差异是代码改动带来的就更新入库产物，不是就说明代码退化了；" \
```

