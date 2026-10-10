# 删除批第 19 批合入的改动原样（2026-10-05）

## 实现员 r：crates.patch（/tmp/claude-1000/m3-batch19/r/patch/crates.patch）

````diff
diff --git a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
index c0949ad..d592a73 100644
--- a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
+++ b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
@@ -3,11 +3,16 @@
 //! 只供测试的旋钮把容器与节点压小（一片 1 条、一个节点 2 条），旋钮不落盘；重建读层级 > 1 的根。池是两块 4 GiB 盘。
 
 mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
 
 use common::{build_pool, file_content, parameters, BuiltPool};
+use common_deletion_in_progress::{
+    pool_with_new_inodes, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+};
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{InodeNumber, InstanceGeneration};
+use singlefs_core::address::InodeNumber;
 use singlefs_core::admission::{
     central_mapping_entries_of_one_lineage_container_cow,
     checkpoint_cost_of_one_lineage_container_cow, MetadataBlocks,
@@ -29,14 +34,11 @@ use singlefs_core::tombstone_tree::{
     TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
 };
 use singlefs_core::transaction::{
-    publish_new_inodes, publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty,
-    PoolWriter, TransactionOutput, TransactionUnit,
+    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, PoolWriter,
+    TransactionOutput, TransactionUnit,
 };
 use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
 
-/// 删除与回退都在新池新建文件那个实例里发。
-const INSTANCE_OF_THE_PUBLISHES: InstanceGeneration = InstanceGeneration(1);
-
 /// 一片容器 1 条、一个码 2 节点 2 条：第 3 片容器根满长一层，第 5 片长到层级 3。
 fn capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
     ContainerTreeCapacities {
@@ -59,28 +61,6 @@ fn position(level: u8, index_in_level: u32) -> ContainerTreeNodePosition {
     }
 }
 
-/// 建空 inode 那次的写入时间；unlink 它们的意图照传这一个（记录现有的时间，盘上字节与改之前相同）。
-const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = 1_788_000_100;
-
-/// 新池新建文件之后建 `count` 个空 inode（号从 2 起）。
-fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {
-    let mut pool = build_pool(tag);
-    let publish_parameters = parameters();
-    let previous = pool.output.clone();
-    let devices = pool.devices.as_mut().expect("镜像还开着");
-    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
-    pool.output = publish_new_inodes(
-        &mut writer,
-        &mut pool.allocator,
-        &previous,
-        count,
-        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
-        INSTANCE_OF_THE_PUBLISHES,
-    )
-    .expect("建空 inode");
-    pool
-}
-
 /// unlink 一个空 inode：意图那次（交回它），再走空（空 inode 没有数据单元，直接是最后一批）。写入口装 `capacities`。
 fn unlink(
     pool: &mut BuiltPool,
diff --git a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
index aad7510..afa5639 100644
--- a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
+++ b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
@@ -12,7 +12,7 @@
 mod common_admission;
 
 use common_admission::{
-    plain_devices_on,
+    content_of, plain_devices_on,
     start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
     start_plain, PoolUnderTest, OVERWRITE_BYTES,
 };
@@ -27,13 +27,13 @@ use singlefs_core::mount::{
     ShadowLedger,
 };
 use singlefs_core::mounted_read::mount_read_only;
-use singlefs_core::mounted_session::MountedSession;
+use singlefs_core::mounted_session::{MountedSession, UserChange};
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
 use singlefs_core::recovery::{recover, JournalPolicy};
 use singlefs_core::root_ring::RootRingSlot;
 use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{
-    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
+    publish_unlink_intent, publish_unlink_step, FirstFile, PoolWriter, FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_core::unlink::{
@@ -569,7 +569,8 @@ fn after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_reading_no_r
 /// 崩了再挂 1 次、再覆盖写 1 次；卸载只落下第一个写就崩（盘 0 的系统配置带新 F = 现行那一版的 txg，盘 1 上最新的根比它低），两块盘都在、
 /// 再可写挂载（影子账关：没有隔离位）做成。挂载交回时分配器每块盘上的空闲槽数与三种位都没置的槽数相同：一个扣住位都没留下。
 /// 判别力：删掉暖机之后那一句放开，重建时释放代高于 m、不高于 F_生效 的那一截（上一次覆盖写换下的槽）留着扣住位交给会话，最后一项红。
-/// 用户看得到的结局没找到（推的，没量过）：扣住的槽准入照算可用、落点取不到时先推抬 F 那一串，那一串做完自己放开全部扣住位，写照样做成。
+/// 用户看得到的结局在窄盘上造出来了，见
+/// [`after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk_the_first_write_of_the_next_writable_mount_regrows_it_without_raising_the_floor`]。
 #[test]
 fn after_a_crashed_unmount_of_a_four_gibibyte_pool_the_next_writable_mount_hands_over_no_held_slot_once_its_warm_up_landed_on_every_device(
 ) {
@@ -600,3 +601,128 @@ fn after_a_crashed_unmount_of_a_four_gibibyte_pool_the_next_writable_mount_hands
         );
     }
 }
+
+/// 卸载之前现行那一版换下的槽（释放代就是那一版的 txg）：卸载半途崩溃（只落下盘 0 的系统配置写）之后，F_生效 是那一版的 txg，
+/// 盘 1 上最新的根早它一个 txg、还引用这几个槽，重建分配器时回收、扣到这次挂载的暖机落满每块盘（`mount::rebuilt_allocator`）。
+fn slots_released_by_the_current_version(pool: &PoolUnderTest<SparseBlockDevice>) -> Vec<u64> {
+    pool.current_file_version()
+        .released
+        .iter()
+        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
+        .collect()
+}
+
+/// 放开挪到暖机之前那一刀（判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节 K1-N4 行「钉不住放开挪到暖机之前」）：
+/// 不补环窗口、准入关着造的窄盘（单元区 176 槽），第一个文件之后不再写；卸载只落下第一个写就崩，两块盘都在、再可写挂载（准入判着、影子账开、
+/// 批大小 1）做成。写行那次与暖机每一次写下的单元一个都不落在卸载之前现行那一版换下的槽上：那几个槽扣到暖机落满每块盘才放开。
+/// 判别力：`mount::establish_instance` 里那一句 `release_reclaim_holds` 挪到暖机之前时，暖机那次的单元写进换下的槽 1410（盘 1 上最新的根还引用它）：
+/// 取号之前在分配器拷贝上预演的落点与真发的不同，挂载在预演对真发那一判上 panic，这一条红；那一判不在时，最后一项红
+/// （在副本里关掉那一判量过：暖机落在换下的槽 [1410] 上）。只读挂载丢盘的那几条都在写行那次就崩，暖机没跑到，判不出这一刀。
+#[test]
+fn after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_until_its_warm_up_landed_on_every_device(
+) {
+    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            device_width,
+            176,
+        );
+    let released_before_the_unmount = slots_released_by_the_current_version(&pool);
+    assert!(
+        !released_before_the_unmount.is_empty(),
+        "卸载之前现行那一版换下过槽（第一个文件那次换下上一版的单元）"
+    );
+    let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);
+    let mut remount_devices = plain_devices_on(&after_the_unmount);
+    let mounted = mount_writable_through(
+        RemountEntry::WithDeletionBatchesOfOne,
+        &pool,
+        &mut remount_devices,
+    )
+    .unwrap_or_else(|error| panic!("卸载半途崩溃之后两块盘都在、再可写挂载：{error:?}"));
+    assert!(
+        !mounted.output.warm_up_publishes.is_empty(),
+        "这次挂载在写行之后推了暖机"
+    );
+    let slots_taken_into_the_released_ones: Vec<u64> = std::iter::once(&mounted.output.row_publish)
+        .chain(&mounted.output.warm_up_publishes)
+        .filter_map(|version| version.version_of_trees())
+        .flat_map(|version| {
+            version
+                .units
+                .iter()
+                .flat_map(|unit| unit.slot.0..unit.slot.0 + unit.identity.span_slots())
+        })
+        .filter(|slot| released_before_the_unmount.contains(slot))
+        .collect();
+    assert_eq!(
+        slots_taken_into_the_released_ones,
+        Vec::<u64>::new(),
+        "写行那次与暖机写下的单元不落在卸载之前现行那一版换下的槽 {released_before_the_unmount:?} 上"
+    );
+}
+
+/// 第一个文件先长到这么多个数据单元、再缩成一个单元：缩的那次换下的槽够多，扣着它们第一次写就长不回去（单元区 176 槽）。
+const DATA_UNITS_OF_THE_FILE_BEFORE_IT_IS_SHRUNK: usize = 36;
+
+/// 暖机之后放开那一句用户看得到的结局（判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第三节第 4 条：`admission.rs`
+/// `PlacementRoomOnOneDevice` 读落点空间时扣住位算挡着）：不补环窗口、准入关着造的窄盘（单元区 176 槽），第一个文件顺序写长到 36 个单元、
+/// 再覆盖写缩成一个单元（换下 36 个数据单元，释放代是这一版的 txg）；卸载只落下第一个写就崩，两块盘都在、再可写挂载（准入关着、影子账开）做成。
+/// 挂载交回之后第一次写把文件顺序写回 36 个单元：一串抬 F 都不推、只发成这一版，F 停在挂载那一刻的值。
+/// 判别力：删掉 `mount::establish_instance` 暖机之后那一句 `release_reclaim_holds` 时，缩的那次换下的槽留着扣住位交给会话，落点取不到，
+/// 这次写拒成空间不够（`NoSpaceAfterRaisingTheFloor`，副本里量过：单元区 176 槽上文件长到 32–40 个单元的每一点都是这个结局，不删时都写成、不推抬 F），这一条红。
+#[test]
+fn after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk_the_first_write_of_the_next_writable_mount_regrows_it_without_raising_the_floor(
+) {
+    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let length_of_the_grown_file =
+        (DATA_UNITS_OF_THE_FILE_BEFORE_IT_IS_SHRUNK - 1) * data_unit_payload_capacity() + 100;
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            device_width,
+            176,
+        );
+    pool.sequential_write(length_of_the_grown_file)
+        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 36 个单元：{refusal:?}"));
+    pool.overwrite(OVERWRITE_BYTES)
+        .unwrap_or_else(|refusal| panic!("覆盖写把文件缩成一个单元：{refusal:?}"));
+    let released_by_the_shrink = slots_released_by_the_current_version(&pool);
+    assert!(
+        released_by_the_shrink.len() >= 2 * DATA_UNITS_OF_THE_FILE_BEFORE_IT_IS_SHRUNK,
+        "缩的那次换下了 36 个数据单元（每个两槽）：换下 {} 槽",
+        released_by_the_shrink.len()
+    );
+    let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);
+    let mut remount_devices = plain_devices_on(&after_the_unmount);
+    let mounted = mount_writable_with_test_only_switches(
+        &pool.parameters,
+        &mut remount_devices,
+        SpaceAdmission::SkippedByTheTestOnlySwitch,
+        ShadowLedger::On,
+    )
+    .unwrap_or_else(|error| panic!("卸载半途崩溃之后两块盘都在、再可写挂载：{error:?}"));
+    let floor_of_the_mount = mounted.current.root().rollback_floor;
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    let write_time_seconds = pool.write_time_seconds + 1;
+    let regrown_content = content_of(length_of_the_grown_file, write_time_seconds);
+    let published = session
+        .publish_user_change(
+            &mut remount_devices,
+            UserChange::SequentialWrite(FirstFile {
+                content: &regrown_content,
+                write_time_seconds,
+            }),
+        )
+        .unwrap_or_else(|refusal| {
+            panic!("挂载交回之后第一次写把文件顺序写回 36 个单元：{refusal:?}")
+        });
+    assert_eq!(
+        (
+            published.floor_raises.len(),
+            published.versions_published.len(),
+            session.current.root().rollback_floor,
+        ),
+        (0, 1, floor_of_the_mount),
+        "第一次写一串抬 F 都不推、只发成这一版，F 停在挂载那一刻的值"
+    );
+}
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
index 5965e56..056c2b5 100644
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -552,37 +552,91 @@ fn time_seconds_of_the_first_file_on_each_device(
 /// 判别力：意图照旧带上一版的时间时第一项红；那一批不照抄上一版这条记录的时间（另取一个）时第二项红。
 #[test]
 fn the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it() {
-    let mut pool = pool_with_a_three_unit_file("write-time-of-the-deletion-steps");
-    let time_before_the_unlink = write_time_seconds_of_the_first_file(&pool);
-    let time_of_the_intent = time_before_the_unlink + 600;
-    let intent = publish_the_intent_in_process_at(&mut pool, time_of_the_intent);
+    let observed = time_fields_on_disk_after_an_intent_and_a_middle_batch_in_process(
+        "write-time-of-the-deletion-steps",
+        GivenTimeAgainstThePreviousVersion::Later,
+    );
+    assert_eq!(
+        observed.on_disk_in_every_version_with_the_record[0],
+        vec![[observed.given_time; 3]; 2],
+        "意图那一版两块盘上那条记录的 atime、mtime、ctime 秒都是给的时间（上一版是 {}）",
+        observed.time_of_the_previous_version
+    );
+    assert_eq!(
+        observed.on_disk_in_every_version_with_the_record[1],
+        vec![[observed.given_time; 3]; 2],
+        "中间那一批照抄意图那次写进去的时间"
+    );
+}
+
+/// 进程内发意图（给一个与这条记录上一版不同的时间，在上一版哪一边由 `given_time_against_the_previous_version` 定），再发一批（留 2）；
+/// 读意图那一版与那一批各自两块盘上第一个文件那条记录的三个时间秒（按先后，一版一项）。
+fn time_fields_on_disk_after_an_intent_and_a_middle_batch_in_process(
+    pool_tag: &str,
+    given_time_against_the_previous_version: GivenTimeAgainstThePreviousVersion,
+) -> TimeFieldsOnDiskAfterAnUnlink {
+    let mut pool = pool_with_a_three_unit_file(pool_tag);
+    let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
+    let given_time =
+        given_time_against_the_previous_version.given_time(time_of_the_previous_version);
+    let intent = publish_the_intent_in_process_at(&mut pool, given_time);
     let on_disk_after_the_intent =
         time_seconds_of_the_first_file_on_each_device(&pool.memory_pool(), &intent);
     let middle_batch = publish_the_next_step_in_process(&mut pool);
     let on_disk_after_the_middle_batch =
         time_seconds_of_the_first_file_on_each_device(&pool.memory_pool(), &middle_batch);
-    assert_eq!(
-        on_disk_after_the_intent,
-        vec![[time_of_the_intent; 3]; 2],
-        "意图那一版两块盘上那条记录的 atime、mtime、ctime 秒都是给的时间（上一版是 {time_before_the_unlink}）"
+    TimeFieldsOnDiskAfterAnUnlink {
+        time_of_the_previous_version,
+        given_time,
+        on_disk_in_every_version_with_the_record: vec![
+            on_disk_after_the_intent,
+            on_disk_after_the_middle_batch,
+        ],
+    }
+}
+
+/// 进程内发意图给的时间早于这条记录上一版（上一版减 600 秒）：意图那一版与中间那一批两块盘上那条记录的三个时间秒都等于给的那个较早的时间。
+/// 上面那一条只有晚于上一版一形，判不出「意图取给的时间与上一版里较大的那个」；会话与续做两条早于上一版的用例不经直接调
+/// `publish_unlink_intent` 这一路。
+/// 判别力：意图取 `write_time_seconds.max(record.write_time_seconds)`（`research/prompts/m3-batch16-code-r1-main-verification.md`
+/// 第二节 N7 攻方那一刀）时意图写下的是上一版的时间，这一条红、晚于上一版那一条照旧绿。
+#[test]
+fn the_intent_given_a_time_earlier_than_the_previous_version_writes_that_earlier_time_into_the_three_time_fields_and_a_middle_batch_copies_it(
+) {
+    let observed = time_fields_on_disk_after_an_intent_and_a_middle_batch_in_process(
+        "earlier-write-time-of-the-deletion-steps",
+        GivenTimeAgainstThePreviousVersion::Earlier,
     );
     assert_eq!(
-        on_disk_after_the_middle_batch,
-        vec![[time_of_the_intent; 3]; 2],
-        "中间那一批照抄意图那次写进去的时间"
+        observed.on_disk_in_every_version_with_the_record,
+        vec![vec![[observed.given_time; 3]; 2]; 2],
+        "意图那一版与中间那一批，两块盘上那条记录的 atime、mtime、ctime 秒都是给的那个较早的时间（上一版是 {}）",
+        observed.time_of_the_previous_version
     );
 }
 
 /// 调用方给 unlink 的时间与这条记录上一版的写入时间差几秒。
 const SECONDS_BETWEEN_THE_GIVEN_TIME_AND_THE_PREVIOUS_VERSION: u64 = 600;
 
-/// 调用方给 unlink 的时间落在这条记录上一版写入时间的哪一边。Q-F 乙只说意图写调用方给的时间、各批照抄，没说时间只进不退；
+/// 给 unlink 的时间比这条记录上一版晚几秒、不是 60 的倍数（上一版是测试池写的固定时间，是 60 的倍数）：取整到分钟的实现写下的不是它。
+const SECONDS_AFTER_THE_PREVIOUS_VERSION_THAT_ARE_NOT_A_WHOLE_MINUTE: u64 = 7;
+
+/// 给 unlink 的时间超过 32 位（2^32 + 7 秒，晚于测试池写的固定时间）：按 32 位存的实现写下的是 7。
+const GIVEN_TIME_PAST_THIRTY_TWO_BITS_IN_SECONDS: u64 = (1 << 32) + 7;
+
+/// 会话里 unlink 之前先建的那个 inode 的时间比这条记录上一版晚几秒：晚于 unlink 给的时间（上一版加 600 秒）。
+const SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_NEW_INODE_BEFORE_THE_UNLINK: u64 = 1000;
+
+/// 调用方给 unlink 的时间怎么取：落在这条记录上一版写入时间的哪一边、取哪个值。Q-F 乙只说意图写调用方给的时间、各批照抄，
+/// 没说时间只进不退、没说存几位、没说精度到哪；每一形判一种错实现（`research/prompts/m3-batch17-code-r1-opus-output.md` 第三节）：
 /// 早于上一版那一形判得出「意图取给的时间与上一版里较大的那个」（`research/prompts/m3-batch16-code-r1-main-verification.md` 第二节 N7），
-/// 晚于上一版那一形判不出。
+/// 超过 32 位那一形判得出「按 32 位存」，不是 60 的倍数那一形判得出「取整到分钟」；晚于上一版 600 秒那一形这三种都判不出。
 #[derive(Clone, Copy)]
 enum GivenTimeAgainstThePreviousVersion {
     Later,
     Earlier,
+    LaterBySecondsThatAreNotAWholeMinute,
+    PastThirtyTwoBits,
 }
 
 impl GivenTimeAgainstThePreviousVersion {
@@ -594,10 +648,22 @@ impl GivenTimeAgainstThePreviousVersion {
             Self::Earlier => time_of_the_previous_version
                 .checked_sub(SECONDS_BETWEEN_THE_GIVEN_TIME_AND_THE_PREVIOUS_VERSION)
                 .expect("上一版的时间是测试池写的固定时间，大于 600 秒"),
+            Self::LaterBySecondsThatAreNotAWholeMinute => time_of_the_previous_version
+                .checked_add(SECONDS_AFTER_THE_PREVIOUS_VERSION_THAT_ARE_NOT_A_WHOLE_MINUTE)
+                .expect("上一版的时间是测试池写的固定时间，加 7 秒不溢出"),
+            Self::PastThirtyTwoBits => GIVEN_TIME_PAST_THIRTY_TWO_BITS_IN_SECONDS,
         }
     }
 }
 
+/// 会话里 unlink 之前这次挂载还发不发别的改动。
+#[derive(Clone, Copy)]
+enum ChangeBeforeTheUnlinkInTheSession {
+    NoOtherChange,
+    /// 先建一个 inode，时间是上一版加 1000 秒（晚于 unlink 给的时间）：会话里时间只进不退的实现把 unlink 的时间抬到它。
+    ANewInodeLaterThanTheGivenTime,
+}
+
 /// 一次 unlink 给的时间、这条记录上一版的写入时间，与之后还带这条记录的几版两块盘上读回的三个时间秒（一版一项、一块盘一组）。
 struct TimeFieldsOnDiskAfterAnUnlink {
     time_of_the_previous_version: u64,
@@ -607,10 +673,11 @@ struct TimeFieldsOnDiskAfterAnUnlink {
 
 /// 经会话发 unlink（`UserChange::Unlink`，会话把调用方给的时间交给意图那次）：给一个与这条记录上一版不同的时间（在上一版哪一边由
 /// `given_time_against_the_previous_version` 定），会话发成意图与三步；读意图那一版与还带这条记录的两批（留 2、留 1）各自两块盘上
-/// 第一个文件那条记录的三个时间秒。
+/// 第一个文件那条记录的三个时间秒。`change_before_the_unlink` 说这次挂载在 unlink 之前还发不发别的改动。
 fn time_fields_on_disk_after_a_session_unlink(
     pool_tag: &str,
     given_time_against_the_previous_version: GivenTimeAgainstThePreviousVersion,
+    change_before_the_unlink: ChangeBeforeTheUnlinkInTheSession,
 ) -> TimeFieldsOnDiskAfterAnUnlink {
     let mut pool = pool_with_a_three_unit_file(pool_tag);
     let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
@@ -618,6 +685,27 @@ fn time_fields_on_disk_after_a_session_unlink(
         given_time_against_the_previous_version.given_time(time_of_the_previous_version);
     let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
     let (_, mut session) = MountedSession::of_the_mount(mounted);
+    match change_before_the_unlink {
+        ChangeBeforeTheUnlinkInTheSession::NoOtherChange => {}
+        ChangeBeforeTheUnlinkInTheSession::ANewInodeLaterThanTheGivenTime => {
+            let time_of_the_new_inode = time_of_the_previous_version
+                .checked_add(SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_NEW_INODE_BEFORE_THE_UNLINK)
+                .expect("上一版的时间是测试池写的固定时间，加 1000 秒不溢出");
+            assert!(
+                time_of_the_new_inode > given_time,
+                "先建的那个 inode 的时间 {time_of_the_new_inode} 晚于 unlink 给的时间 {given_time}"
+            );
+            session
+                .publish_user_change(
+                    &mut devices,
+                    UserChange::NewInodes {
+                        count: 1,
+                        write_time_seconds: time_of_the_new_inode,
+                    },
+                )
+                .expect("同一次挂载里 unlink 之前先建一个 inode");
+        }
+    }
     let published = session
         .publish_user_change(
             &mut devices,
@@ -651,6 +739,7 @@ fn a_session_unlink_writes_the_given_time_into_the_three_time_fields_of_the_inte
     let observed = time_fields_on_disk_after_a_session_unlink(
         "write-time-of-a-session-unlink",
         GivenTimeAgainstThePreviousVersion::Later,
+        ChangeBeforeTheUnlinkInTheSession::NoOtherChange,
     );
     assert_eq!(
         observed.on_disk_in_every_version_with_the_record,
@@ -670,6 +759,7 @@ fn a_session_unlink_given_a_time_earlier_than_the_previous_version_writes_that_e
     let observed = time_fields_on_disk_after_a_session_unlink(
         "earlier-write-time-of-a-session-unlink",
         GivenTimeAgainstThePreviousVersion::Earlier,
+        ChangeBeforeTheUnlinkInTheSession::NoOtherChange,
     );
     assert_eq!(
         observed.on_disk_in_every_version_with_the_record,
@@ -679,6 +769,74 @@ fn a_session_unlink_given_a_time_earlier_than_the_previous_version_writes_that_e
     );
 }
 
+/// 同一次挂载里先建一个 inode（时间是上一版加 1000 秒），再 unlink 第一个文件、给上一版加 600 秒（晚于上一版、早于会话里上一次给的）：
+/// 意图那一版与留 2、留 1 两批两块盘上那条记录的三个时间秒都等于这次给的时间。上面几条的 unlink 都是这次挂载的第一次改动。
+/// 判别力：会话那一层 unlink 的时间取给的与会话里给过的更晚时间里较大的那个（时间只进不退，攻方 `research/prompts/m3-batch17-code-r1-opus-output.md`
+/// 第三节 qfm）时写下的是上一版加 1000 秒，这一条红；会话里只有 unlink 那几条照旧绿。
+#[test]
+fn a_session_unlink_after_a_new_inode_with_a_later_time_in_the_same_mount_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
+) {
+    let observed = time_fields_on_disk_after_a_session_unlink(
+        "write-time-of-a-session-unlink-after-a-later-new-inode",
+        GivenTimeAgainstThePreviousVersion::Later,
+        ChangeBeforeTheUnlinkInTheSession::ANewInodeLaterThanTheGivenTime,
+    );
+    assert_eq!(
+        observed.on_disk_in_every_version_with_the_record,
+        vec![vec![[observed.given_time; 3]; 2]; 3],
+        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是这次给的时间，不是之前那个 inode 的更晚时间（上一版是 {}）",
+        observed.time_of_the_previous_version
+    );
+}
+
+/// 给 unlink 的时间超过 32 位（2^32 + 7 秒）：意图那一版与留 2、留 1 两批两块盘上那条记录的三个时间秒都是整个 64 位值。
+/// 判别力：意图写时间时按 32 位存（攻方第三节 qfu）时写下的是 7，这一条红；给的时间小于 2^32 的几条照旧绿。
+#[test]
+fn a_session_unlink_given_a_time_past_thirty_two_bits_writes_it_whole_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
+) {
+    let observed = time_fields_on_disk_after_a_session_unlink(
+        "write-time-of-a-session-unlink-past-thirty-two-bits",
+        GivenTimeAgainstThePreviousVersion::PastThirtyTwoBits,
+        ChangeBeforeTheUnlinkInTheSession::NoOtherChange,
+    );
+    assert!(
+        observed.given_time > u64::from(u32::MAX)
+            && observed.given_time > observed.time_of_the_previous_version,
+        "给的时间 {} 超过 32 位、晚于上一版 {}",
+        observed.given_time,
+        observed.time_of_the_previous_version
+    );
+    assert_eq!(
+        observed.on_disk_in_every_version_with_the_record,
+        vec![vec![[observed.given_time; 3]; 2]; 3],
+        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是给的整个 64 位时间"
+    );
+}
+
+/// 给 unlink 的时间不是 60 的倍数（上一版加 7 秒）：意图那一版与留 2、留 1 两批两块盘上那条记录的三个时间秒精确到秒、等于给的时间。
+/// 判别力：意图写时间时取整到分钟（攻方第三节 qfr）时写下的是上一版的时间，这一条红；给的时间是 60 的倍数的几条照旧绿。
+#[test]
+fn a_session_unlink_given_a_time_that_is_not_a_whole_minute_writes_it_to_the_second_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
+) {
+    let observed = time_fields_on_disk_after_a_session_unlink(
+        "write-time-of-a-session-unlink-not-a-whole-minute",
+        GivenTimeAgainstThePreviousVersion::LaterBySecondsThatAreNotAWholeMinute,
+        ChangeBeforeTheUnlinkInTheSession::NoOtherChange,
+    );
+    assert_ne!(
+        observed.given_time % 60,
+        0,
+        "给的时间 {} 不是 60 的倍数",
+        observed.given_time
+    );
+    assert_eq!(
+        observed.on_disk_in_every_version_with_the_record,
+        vec![vec![[observed.given_time; 3]; 2]; 3],
+        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是给的时间、精确到秒（上一版是 {}）",
+        observed.time_of_the_previous_version
+    );
+}
+
 /// 进程内发意图（给一个与这条记录上一版不同的时间，在上一版哪一边由 `given_time_against_the_previous_version` 定），一批没发就重开镜像
 /// （意图落盘之后崩溃），可写挂载续做；读续做里还带这条记录的那两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒。
 fn time_fields_on_disk_in_the_continuation_batches_after_an_intent(
diff --git a/crates/singlefs-harness/tests/unlink_single_file_publishes.rs b/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
index 56aba57..5c6ec29 100644
--- a/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
+++ b/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
@@ -2,14 +2,17 @@
 //! 意图那次、不是最后一批的一批、最后一批。三个单元的文件按只供测试的批大小 1 删：意图 → 留 2 → 留 1 → 最后一批。
 
 mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
 
-use common::{
-    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
-    BuiltPool, FIXED_WRITE_TIME_SECONDS,
+use common::{parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
+use common_deletion_in_progress::{
+    batches_of_one_data_unit, pool_with_a_three_unit_file, publish_the_next_step_in_batches_of_one,
+    FIRST_FILE, INSTANCE_OF_THE_PUBLISHES,
 };
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
+use singlefs_core::address::CheckpointTxg;
 use singlefs_core::allocator::Placement;
 use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
 use singlefs_core::pointer::NodePointer;
@@ -20,44 +23,13 @@ use singlefs_core::recovery::{
     rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
 };
 use singlefs_core::transaction::{
-    publish_overwrite, publish_unlink_intent, publish_unlink_step,
-    publish_unlink_steps_until_the_queue_is_empty, FirstFile, PoolWriter, PublishError,
-    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
+    publish_overwrite, publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty,
+    FirstFile, PoolWriter, PublishError, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::{data_unit_payload_capacity, PACKED_TYPE_TOMBSTONE};
-use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
 use singlefs_harness::memory_pool::MemoryPool;
 
-const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
-
-/// 删除的那一路都在新池新建文件那个实例里发。
-const INSTANCE_OF_THE_PUBLISHES: InstanceGeneration = InstanceGeneration(1);
-
-fn batches_of_one_data_unit() -> UnlinkBatchSize {
-    UnlinkBatchSize::CappedForTests(
-        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
-    )
-}
-
-/// 三个数据单元的内容：最后一个单元装半个净荷。
-fn content_of_three_data_units() -> Vec<u8> {
-    let payload_capacity = data_unit_payload_capacity();
-    (0..2 * payload_capacity + payload_capacity / 2)
-        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
-        .collect()
-}
-
-/// 新池新建文件之后把第一个文件顺序写成三个数据单元。
-fn pool_with_a_three_unit_file(tag: &str) -> BuiltPool {
-    let mut pool = build_pool(tag);
-    let grown = sequential_write_in_process_as_instance_one_and_advance_the_pool(
-        &mut pool,
-        &content_of_three_data_units(),
-    );
-    assert_eq!(grown.data_pointers.len(), 3, "第一个文件三个数据单元");
-    pool
-}
-
+/// 意图那次被拒要交回错（已经删完的那一格），`common_deletion_in_progress::publish_the_intent` 被拒就 panic：这一份照旧留在这里。
 fn publish_the_intent(pool: &mut BuiltPool) -> Result<TransactionOutput, PublishError> {
     let publish_parameters = parameters();
     let previous = pool.output.clone();
@@ -79,24 +51,6 @@ fn publish_the_intent(pool: &mut BuiltPool) -> Result<TransactionOutput, Publish
     Ok(output)
 }
 
-fn publish_the_next_step_in_batches_of_one(pool: &mut BuiltPool) -> TransactionOutput {
-    let publish_parameters = parameters();
-    let previous = pool.output.clone();
-    let devices = pool.devices.as_mut().expect("镜像还开着");
-    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
-    let output = publish_unlink_step(
-        &mut writer,
-        &mut pool.allocator,
-        &previous,
-        FIRST_FILE,
-        batches_of_one_data_unit(),
-        INSTANCE_OF_THE_PUBLISHES,
-    )
-    .expect("删除的下一步");
-    pool.output = output.clone();
-    output
-}
-
 fn tags_of(roles: &[TransactionUnit]) -> Vec<String> {
     roles.iter().map(|role| role.tag()).collect()
 }
diff --git a/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs b/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
new file mode 100644
index 0000000..fad78e4
--- /dev/null
+++ b/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
@@ -0,0 +1,182 @@
+//! unlink 单次发布的写入量不随文件长（`.claude/kb/checks-owed.md` C2（无界操作做成单事务），用户 2026-10-05 同意把 C2 的对象补上 unlink；
+//! D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」）：几份单元数不同的文件（1、144、145、432 个数据单元，跨过一批 144 个的边界）
+//! 各按产品批大小（`UnlinkBatchSize::OneExtentLowerLeaf`）发意图、走完全部批，量每一次发布（意图、每一批）交给设备的写调用与字节
+//! （`TransactionOutput::writes`，发布路径逐个写调用记的账），与每一次换下的数据单元个数。
+
+mod common;
+
+use common::{
+    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
+};
+use singlefs_core::address::{InodeNumber, InstanceGeneration};
+use singlefs_core::transaction::{
+    publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::UnlinkBatchSize;
+use singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
+
+const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
+
+/// 新池新建文件那个实例：意图与各批都在这个进程里发。
+const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);
+
+/// 最大一份文件里写调用最多的那次发布，至多比一个单元的文件里写调用最多的那次多这么多个写调用（推的，逐项见下，没逐项核到代码里每一处）：
+/// 每个单元两块盘各写一份（`admission::UnitsOfAPublishToLand` 的文档：第一版每个单元落池里每一块盘），一批走的那条路径上比一个单元的文件
+/// 至多多三个节点——文件过了一片下段叶（144 个单元）extent 树多一层上段（D8（核心索引结构） 已定项 16 第 2 条的 K 取一片下段叶的单元数）、
+/// 一批换下的 144 个单元的记录与映射条目各至多多罩一片分配记录树叶与一片中央映射树叶。2 × 3 = 6。
+/// 不随文件的单元数长：432 个单元的文件与 145 个单元的文件，一批走的路径一样长、罩的叶一样多。
+const WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE: u64 = 6;
+
+/// unlink 的一次发布交给设备的写，与这次换下的数据单元个数。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct WritesOfOnePublish {
+    write_calls: u64,
+    written_bytes: u64,
+    data_units_released: usize,
+}
+
+fn writes_of_one_publish(
+    previous: &TransactionOutput,
+    published: &TransactionOutput,
+) -> WritesOfOnePublish {
+    let total = published.writes.total();
+    WritesOfOnePublish {
+        write_calls: total.write_calls,
+        written_bytes: total.written_bytes,
+        data_units_released: previous
+            .data_pointers
+            .len()
+            .checked_sub(published.data_pointers.len())
+            .expect("删除的每一步只减不增数据单元"),
+    }
+}
+
+/// 新池新建文件之后把第一个文件顺序写成 `data_units_of_the_file` 个数据单元（最后一个装半个净荷），按产品批大小发意图、
+/// 走完全部批；交回每一次发布的写，按先后（第一项是意图那次）。
+fn writes_of_every_publish_of_an_unlink_in_product_batches(
+    data_units_of_the_file: usize,
+) -> Vec<WritesOfOnePublish> {
+    let payload_capacity = data_unit_payload_capacity();
+    let content: Vec<u8> = (0..(data_units_of_the_file - 1) * payload_capacity
+        + payload_capacity / 2)
+        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
+        .collect();
+    let mut pool = build_pool(&format!(
+        "unlink-writes-per-publish-of-{data_units_of_the_file}-data-units"
+    ));
+    let grown =
+        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
+    assert_eq!(grown.data_pointers.len(), data_units_of_the_file);
+    let publish_parameters = parameters();
+    let mut writes = Vec::new();
+    {
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        pool.output = publish_unlink_intent(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            FIRST_FILE,
+            previous
+                .inode_record
+                .expect("意图之前第一个文件那条记录在")
+                .write_time_seconds,
+            INSTANCE_OF_THE_PROCESS,
+        )
+        .expect("意图");
+        writes.push(writes_of_one_publish(&previous, &pool.output));
+    }
+    // 迭代上界：每一步至少删一个单元，文件 `data_units_of_the_file` 个单元。
+    while !pool.output.deleted_inodes.is_empty() {
+        assert!(writes.len() <= data_units_of_the_file + 1, "删除步数有界");
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        pool.output = publish_unlink_step(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            FIRST_FILE,
+            UnlinkBatchSize::OneExtentLowerLeaf,
+            INSTANCE_OF_THE_PROCESS,
+        )
+        .expect("删除的下一步");
+        writes.push(writes_of_one_publish(&previous, &pool.output));
+    }
+    writes
+}
+
+fn write_calls_of_every_publish(writes: &[WritesOfOnePublish]) -> Vec<u64> {
+    writes.iter().map(|publish| publish.write_calls).collect()
+}
+
+fn largest_write_calls_of_one_publish(writes: &[WritesOfOnePublish]) -> u64 {
+    writes
+        .iter()
+        .map(|publish| publish.write_calls)
+        .max()
+        .expect("意图那次至少一次发布")
+}
+
+/// 四份文件（1、144、145、432 个数据单元）：
+/// ① 每一次发布换下的数据单元不多于一批（144 个，`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）；
+/// ② 每份文件里写调用最多的那次发布，至多比一个单元的文件里的多 [`WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE`]；
+/// ③ 每一次发布的写调用钉成绝对值（发布路径今天写出的形状：一个单元的文件意图 31、最后一批 25；432 个单元的文件意图 33、三批 37、35、33）。
+/// 判别力：批大小改成整个文件一批（`UnlinkBatchSize::data_units_per_batch` 的产品那一臂取 `u32::MAX`）时，145 与 432 个单元的文件
+/// 意图之后只有一批、换下 145 与 432 个单元，①③ 红。这几个规模上整个文件一批的单次写调用不比分批的多（副本里量过：432 个单元的文件
+/// 那一批 29 次，分批时最多 37 次），② 判不出这一刀；要到 2304 个单元以上整个文件一批的单次写调用才超过分批的（副本里 release 量过），
+/// 那几个规模单跑十几秒到几十秒，没进这一条。
+// harness-test-granularity:one-scenario 斜率要几份不同单元数的文件一起比，拆开就没有斜率
+#[test]
+fn unlink_of_files_of_one_to_three_batches_releases_at_most_one_batch_per_publish_and_its_largest_publish_does_not_grow_with_the_file(
+) {
+    let one_data_unit = writes_of_every_publish_of_an_unlink_in_product_batches(1);
+    let one_batch = writes_of_every_publish_of_an_unlink_in_product_batches(144);
+    let one_batch_and_one_unit = writes_of_every_publish_of_an_unlink_in_product_batches(145);
+    let three_batches = writes_of_every_publish_of_an_unlink_in_product_batches(432);
+    let every_file = [
+        (1, &one_data_unit),
+        (144, &one_batch),
+        (145, &one_batch_and_one_unit),
+        (432, &three_batches),
+    ];
+    for (data_units_of_the_file, writes) in every_file {
+        println!("{data_units_of_the_file} 个数据单元的文件，每一次发布（意图、各批）：{writes:?}");
+    }
+    for (data_units_of_the_file, writes) in every_file {
+        assert!(
+            writes.iter().all(|publish| u64::try_from(publish.data_units_released)
+                .expect("单元数装得进 u64")
+                <= EXTENT_TREE_LOWER_LEAF_DATA_UNITS),
+            "{data_units_of_the_file} 个数据单元的文件，每一次发布换下的数据单元不多于一批 {EXTENT_TREE_LOWER_LEAF_DATA_UNITS} 个：{writes:?}"
+        );
+    }
+    let largest_of_the_one_unit_file = largest_write_calls_of_one_publish(&one_data_unit);
+    for (data_units_of_the_file, writes) in every_file {
+        assert!(
+            largest_write_calls_of_one_publish(writes)
+                <= largest_of_the_one_unit_file
+                    + WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE,
+            "{data_units_of_the_file} 个数据单元的文件，写调用最多的那次发布 {} 次，至多比一个单元的文件的 {largest_of_the_one_unit_file} 次多 \
+             {WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE} 次",
+            largest_write_calls_of_one_publish(writes)
+        );
+    }
+    assert_eq!(
+        [
+            write_calls_of_every_publish(&one_data_unit),
+            write_calls_of_every_publish(&one_batch),
+            write_calls_of_every_publish(&one_batch_and_one_unit),
+            write_calls_of_every_publish(&three_batches),
+        ],
+        [
+            vec![31, 25],
+            vec![31, 29],
+            vec![31, 31, 29],
+            vec![33, 37, 35, 33],
+        ],
+        "1、144、145、432 个数据单元的文件，每一次发布（意图、各批）的写调用"
+    );
+}
````

### r：mutations-append.tsv

````
删除批第 19 批 Q-F qfm：会话那一层 unlink 给意图的时间取给的与现行那一版里各条 inode 记录最晚的写入时间里较大的那个（会话里时间只进不退的单处写法，攻方 m3-batch17-code-r1 qfm），同一次挂载里先建一个更晚的 inode 再 unlink	crates/singlefs-core/src/mounted_session.rs	            inode,\n            write_time_seconds,\n            instance,\n        ),	            inode,\n            write_time_seconds.max(\n                previous\n                    .inode_leaf_containers\n                    .iter()\n                    .flat_map(|container| container.contents.records.iter())\n                    .map(|record| record.write_time_seconds)\n                    .max()\n                    .unwrap_or(0),\n            ),\n            instance,\n        ),	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- a_session_unlink_after_a_new_inode	a_session_unlink_after_a_new_inode_with_a_later_time_in_the_same_mount_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk
删除批第 19 批 Q-F qfu：unlink 的意图写时间时按 32 位存（攻方 m3-batch17-code-r1 qfu），给 2^32 + 7 秒	crates/singlefs-core/src/transaction.rs	                change_count: change_count_of_the_intent,\n                write_time_seconds,	                change_count: change_count_of_the_intent,\n                write_time_seconds: u64::from(write_time_seconds as u32),	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- a_session_unlink_given_a_time_past_thirty_two_bits	a_session_unlink_given_a_time_past_thirty_two_bits_writes_it_whole_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk
删除批第 19 批 Q-F qfr：unlink 的意图写时间时取整到分钟（攻方 m3-batch17-code-r1 qfr），给上一版加 7 秒	crates/singlefs-core/src/transaction.rs	                change_count: change_count_of_the_intent,\n                write_time_seconds,	                change_count: change_count_of_the_intent,\n                write_time_seconds: write_time_seconds / 60 * 60,	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- a_session_unlink_given_a_time_that_is_not_a_whole_minute	a_session_unlink_given_a_time_that_is_not_a_whole_minute_writes_it_to_the_second_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk
删除批第 19 批 N7：unlink 的意图取给的时间与这条记录上一版里较大的那个（时间只进不退，攻方 n7-clamp），进程内直接发意图那一形给早于上一版的时间	crates/singlefs-core/src/transaction.rs	                change_count: change_count_of_the_intent,\n                write_time_seconds,	                change_count: change_count_of_the_intent,\n                write_time_seconds: write_time_seconds.max(record.write_time_seconds),	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- the_intent_given_a_time_earlier_than_the_previous_version	the_intent_given_a_time_earlier_than_the_previous_version_writes_that_earlier_time_into_the_three_time_fields_and_a_middle_batch_copies_it
删除批第 19 批 甲：可写挂载把重建分配器时扣住的那一截挪到暖机之前放开（establish_instance 暖机循环之前先 release_reclaim_holds），暖机的单元写进卸载之前现行那一版换下的槽	crates/singlefs-core/src/mount.rs	    let mut current = row_publish.clone();\n    let mut warm_up_publishes = Vec::new();	    allocator.release_reclaim_holds();\n    let mut current = row_publish.clone();\n    let mut warm_up_publishes = Vec::new();	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_writes_no_unit	after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_until_its_warm_up_landed_on_every_device
删除批第 19 批 N4：可写挂载暖机落满之后不放开扣住的那一截（删掉 establish_instance 暖机之后那一句 release_reclaim_holds），窄盘上挂载交回之后第一次写落点取不到、拒成空间不够	crates/singlefs-core/src/mount.rs	    // 扣住的那一截不交给任何会话。\n    allocator.release_reclaim_holds();	    // 扣住的那一截不交给任何会话。	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk	after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk_the_first_write_of_the_next_writable_mount_regrows_it_without_raising_the_floor
删除批第 19 批 C2：unlink 的产品批大小改成整个文件一批（data_units_per_batch 的产品那一臂取 u32::MAX），一次发布换下整个文件的数据单元	crates/singlefs-core/src/unlink.rs	            UnlinkBatchSize::OneExtentLowerLeaf => EXTENT_TREE_LOWER_LEAF_DATA_UNITS,	            UnlinkBatchSize::OneExtentLowerLeaf => u32::MAX as u64,	-p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file	unlink_of_files_of_one_to_three_batches_releases_at_most_one_batch_per_publish_and_its_largest_publish_does_not_grow_with_the_file
````

## 实现员 s：crates.patch（/tmp/claude-1000/m3-batch19/s/patch/crates.patch）

````diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 19de0f1..dd93075 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -206,6 +206,78 @@ impl JournalRecordNamedEntryCapacity {
     }
 }
 
+/// 只供测试压回去的 inode 号水位：这次发布从这个号起发、记账里写它加这次新建的个数（`.claude/kb/checks-owed.md` C479：
+/// 产品路径上号只从记账里的水位发、水位只增，「复用一个已发过的号」这条写路径造不出来；要它造出来，池级 checker 的 I-9.6
+/// （水位大于两处最大号）才有对象可判）。至少是第一个 inode 号（0 是无效 inode 号，D8（核心索引结构） 已定项 6）；
+/// 是不是真的「压回去」（小于上一版的水位）要对着上一版判，在发布路径读它的那一处断言（[`InodeNumberWatermarkSource::watermark_this_publish_starts_from`]）。
+/// 只住内存，不落盘。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct InodeNumberWatermarkPressedBackForTests(u64);
+
+impl InodeNumberWatermarkPressedBackForTests {
+    /// 小于第一个 inode 号（`FIRST_INODE_NUMBER`）交回 `None`。
+    #[must_use]
+    pub fn new(next_inode_number: u64) -> Option<Self> {
+        (next_inode_number >= FIRST_INODE_NUMBER).then_some(Self(next_inode_number))
+    }
+
+    #[must_use]
+    pub const fn next_inode_number(self) -> u64 {
+        self.0
+    }
+}
+
+/// 一次发布从哪个 inode 号水位起算（D5（快照 / 空间记账机制） 已定项 4 第 12 项「inode 号水位」）：产品路径恒
+/// `FromThePreviousVersionsAccounting`；只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）换成压回去的那个号。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum InodeNumberWatermarkSource {
+    /// 上一版记账树里真写下的那一行（[`TransactionOutput::inode_number_watermark`]）。
+    FromThePreviousVersionsAccounting,
+    PressedBackForTests(InodeNumberWatermarkPressedBackForTests),
+}
+
+impl InodeNumberWatermarkSource {
+    /// 走的是哪一臂（五条硬要求第 4 条：分支必须可观测）。
+    #[must_use]
+    pub const fn branch_name(self) -> &'static str {
+        match self {
+            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => {
+                "inode_number_watermark=from_the_previous_versions_accounting"
+            }
+            InodeNumberWatermarkSource::PressedBackForTests(_) => {
+                "inode_number_watermark=pressed_back_for_tests"
+            }
+        }
+    }
+
+    /// 这次发布从哪个号起发：产品路径就是上一版记账里的水位，开关装着时是压回去的那个号。
+    ///
+    /// # Panics
+    /// 开关压的号不小于上一版的水位：那不是「压回一个已发过的号」，开关给错了。在发布路径读它的两处（建 inode 的起号、
+    /// 记账里写的水位）都在任何落盘动作之前。
+    #[must_use]
+    pub fn watermark_this_publish_starts_from(
+        self,
+        inode_number_watermark_of_the_previous_version: u64,
+    ) -> u64 {
+        match self {
+            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => {
+                inode_number_watermark_of_the_previous_version
+            }
+            InodeNumberWatermarkSource::PressedBackForTests(pressed_back) => {
+                assert!(
+                    pressed_back.next_inode_number()
+                        < inode_number_watermark_of_the_previous_version,
+                    "开关把 inode 号水位压到 {}，上一版的水位是 {inode_number_watermark_of_the_previous_version}：\
+                     要小于它才是压回一个已发过的号",
+                    pressed_back.next_inode_number()
+                );
+                pressed_back.next_inode_number()
+            }
+        }
+    }
+}
+
 /// 一个池的写入口：几块盘 + mkfs 参数（几何与区域归属都从这里取）。
 pub struct PoolWriter<'pool, Device: BlockDevice> {
     pub parameters: &'pool MakeFilesystemParameters,
@@ -225,6 +297,8 @@ pub struct PoolWriter<'pool, Device: BlockDevice> {
     /// 墓碑树与谱系树的三个容量（一片墓碑容器、一片谱系容器装满几条，一个码 2 节点装满几条），只供测试的开关
     /// （D18（块里携带什么信息） 已定项 24 第 3 条：旋钮不落盘）；新开的写入口按格式起步。
     container_tree_capacities: ContainerTreeCapacities,
+    /// 这次发布从哪个 inode 号水位起算，只供测试的开关（[`InodeNumberWatermarkSource`]）；新开的写入口按上一版的记账起步。
+    inode_number_watermark_source: InodeNumberWatermarkSource,
 }
 
 impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
@@ -243,6 +317,8 @@ impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
             journal_record_named_entry_capacity:
                 JournalRecordNamedEntryCapacity::FromTheRecordFormat,
             container_tree_capacities: ContainerTreeCapacities::FROM_THE_FORMAT,
+            inode_number_watermark_source:
+                InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,
         }
     }
 }
@@ -310,6 +386,19 @@ impl<Device: BlockDevice> PoolWriter<'_, Device> {
         self.container_tree_capacities
     }
 
+    /// 装上「这次发布从哪个 inode 号水位起算」这个只供测试的开关（[`InodeNumberWatermarkSource`]）。产品路径一处都不调它。
+    /// 装着时这个写入口发的每一次有上一版的发布都从压回去的号起算（建 inode 从它起发号，记账里写它加这次新建的个数）；
+    /// 没有上一版的那次（新池新建文件，水位恒 2）不读上一版的水位，开关不管它。
+    pub fn set_inode_number_watermark_source(&mut self, source: InodeNumberWatermarkSource) {
+        self.inode_number_watermark_source = source;
+    }
+
+    /// 这会儿装着的 inode 号水位来源：运行时看得出走的是哪一条分支。
+    #[must_use]
+    pub fn inode_number_watermark_source(&self) -> InodeNumberWatermarkSource {
+        self.inode_number_watermark_source
+    }
+
     pub fn perform_commit_step(&mut self, step: CommitStep<'_>) -> Result<(), BlockDeviceError> {
         if !matches!(step, CommitStep::Barrier) {
             self.has_writes_since_barrier = true;
@@ -5794,6 +5883,10 @@ pub fn publish_new_inodes<Device: BlockDevice>(
 ) -> Result<TransactionOutput, PublishError> {
     let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
     let first_new_inode_number = previous.inode_number_watermark();
+    // 只供测试的开关装着时从压回去的那个号起发（C479）；记账里写的水位在 `publish_admitted` 里读同一个开关、同一份上一版。
+    let first_new_inode_number = pool
+        .inode_number_watermark_source()
+        .watermark_this_publish_starts_from(first_new_inode_number);
     // 改动计数与事务号都先取成局部量：`crates/mutations.tsv` 有几条变异按 `change_count: txg.0,` 与
     // `transaction: previous.highest_transaction_number_in_this_instance + 1,` 这两串字面锚在 `publish_overwrite` 上，
     // 同一份文件里再出现一次它们就腐化（门禁 59 号；`publish_first_file` 里那句注释是同一件事）。
@@ -7911,7 +8004,10 @@ fn publish_admitted<Device: BlockDevice>(
     let new_inodes_of_this_publish =
         u64::try_from(plan.new_inode_records.len()).expect("这次新建的 inode 数");
     let inode_number_watermark = match previous {
-        Some(previous_version) => previous_version.inode_number_watermark(),
+        // 只供测试的开关装着时从压回去的那个号起算（C479），与 `publish_new_inodes` 起号读的是同一个开关、同一份上一版。
+        Some(previous_version) => pool
+            .inode_number_watermark_source()
+            .watermark_this_publish_starts_from(previous_version.inode_number_watermark()),
         // 新池新建文件：树里只有 inode 1，下一个可用号是 2（字节表六那一行）。
         None => FIRST_INODE_NUMBER + 1,
     } + new_inodes_of_this_publish;
@@ -9310,4 +9406,29 @@ mod tests {
             vec![0]
         );
     }
+
+    #[test]
+    fn the_pressed_back_inode_number_watermark_refuses_the_invalid_inode_number_zero() {
+        assert_eq!(
+            InodeNumberWatermarkPressedBackForTests::new(0),
+            None,
+            "0 是无效 inode 号，不是一个发过的号"
+        );
+        assert_eq!(
+            InodeNumberWatermarkPressedBackForTests::new(FIRST_INODE_NUMBER)
+                .map(InodeNumberWatermarkPressedBackForTests::next_inode_number),
+            Some(FIRST_INODE_NUMBER),
+            "第一个 inode 号压得到"
+        );
+    }
+
+    #[test]
+    #[should_panic(expected = "要小于它才是压回一个已发过的号")]
+    fn a_pressed_back_inode_number_watermark_not_below_the_previous_watermark_is_a_misused_switch()
+    {
+        let not_pressed_back = InodeNumberWatermarkSource::PressedBackForTests(
+            InodeNumberWatermarkPressedBackForTests::new(5).expect("5 不小于第一个 inode 号"),
+        );
+        let _ = not_pressed_back.watermark_this_publish_starts_from(5);
+    }
 }
diff --git a/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs b/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
new file mode 100644
index 0000000..6002fe1
--- /dev/null
+++ b/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
@@ -0,0 +1,130 @@
+//! inode 号水位被只供测试的开关压回一个已发过的号（`.claude/kb/checks-owed.md` C479）：产品路径上号只从记账里的水位发、
+//! 水位只增，「复用一个已发过的号」造不出来；开关（`PoolWriter::set_inode_number_watermark_source`）把这次发布的起号压回去，
+//! 再建一个 inode，看池级 checker 的 I-9.6（水位大于两处最大号）拦不拦得住。
+//! 判别力自证：同一段流把开关关掉，I-9.6 由红转绿。
+//! 「水位压回去、但那个号原来的记录已被换掉」那一形（压回的正是发过的最大号：它的记录还在树里被原地换掉，或已 unlink 删完、
+//! 墓碑在所选根的墓碑树里）I-9.6 拦不住，这里不钉，交主 agent 定要不要另立一条不变量（C479 那一行）。
+
+mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
+
+use common::{parameters, BuiltPool};
+use common_deletion_in_progress::{
+    pool_with_new_inodes, verdict_of, violated_invariants, INSTANCE_OF_THE_PUBLISHES,
+    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_core::transaction::{
+    publish_new_inodes, InodeNumberWatermarkPressedBackForTests, InodeNumberWatermarkSource,
+    PoolWriter, TransactionOutput,
+};
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// 接着现行那一版建一个 inode，写入口上装着 `source` 这个水位来源；交回这一版与它的镜像。
+fn create_one_inode_with_the_watermark_from(
+    pool: &mut BuiltPool,
+    source: InodeNumberWatermarkSource,
+) -> (TransactionOutput, MemoryPool) {
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    writer.set_inode_number_watermark_source(source);
+    assert_eq!(
+        writer.inode_number_watermark_source().branch_name(),
+        source.branch_name(),
+        "运行时看得出走的是哪一臂"
+    );
+    let output = publish_new_inodes(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        1,
+        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+        INSTANCE_OF_THE_PUBLISHES,
+    )
+    .expect("建一个 inode");
+    pool.output = output.clone();
+    (output, pool.memory_pool())
+}
+
+fn pressed_back_to(next_inode_number: u64) -> InodeNumberWatermarkSource {
+    InodeNumberWatermarkSource::PressedBackForTests(
+        InodeNumberWatermarkPressedBackForTests::new(next_inode_number)
+            .expect("不小于第一个 inode 号"),
+    )
+}
+
+/// 这一版 inode 树里 `inode` 那条记录的对象出生代；没有这条记录交回 `None`。
+fn object_birth_of(output: &TransactionOutput, inode: u64) -> Option<u64> {
+    output
+        .inode_leaf_container_contents()
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .find(|record| record.inode == inode)
+        .map(|record| record.object_birth.0)
+}
+
+/// 新池新建文件之后建 inode 2、3、4（水位 5），再建一个 inode：开关把水位压回 3 时新建的那个拿到 3 号——
+/// inode 树里原来那条 3 号记录被原地换成新对象（对象出生代换成这次发布的 txg），记账里的水位写成 4，不大于树里最大的 key 4，
+/// I-9.6 判红；同一段流开关关着时新建的拿 5 号、水位 6，I-9.6 成立、一条违例都没有。
+#[test]
+fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant(
+) {
+    let mut pressed_pool = pool_with_new_inodes("watermark-pressed-back-below-the-largest-key", 3);
+    assert_eq!(
+        pressed_pool.output.inode_number_watermark(),
+        5,
+        "inode 1 到 4 发过，水位 5"
+    );
+    let (pressed, pressed_image) =
+        create_one_inode_with_the_watermark_from(&mut pressed_pool, pressed_back_to(3));
+    assert_eq!(
+        pressed.inode_number_watermark(),
+        4,
+        "从压回去的 3 起发一个，水位写成 4"
+    );
+    assert_eq!(
+        object_birth_of(&pressed, 3),
+        Some(pressed.root.checkpoint_txg.0),
+        "3 号被重发：树里那条记录换成这次发布建的新对象"
+    );
+    match verdict_of(&pressed_image, "I-9.6") {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains("水位 4 不大于 inode 树内最大 key 4"),
+            "违例说明写出树内最大 key 那一半：{detail}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("水位压回已发过的号之后 I-9.6 应当判红，实际 {other_verdict:?}")
+        }
+    }
+    assert_eq!(
+        violated_invariants(&pressed_image),
+        vec!["I-9.6"],
+        "只红 I-9.6"
+    );
+
+    let mut control_pool = pool_with_new_inodes("watermark-not-pressed-back-control", 3);
+    let (control, control_image) = create_one_inode_with_the_watermark_from(
+        &mut control_pool,
+        InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,
+    );
+    assert_eq!(
+        control.inode_number_watermark(),
+        6,
+        "开关关着：发 5 号，水位 6"
+    );
+    assert_eq!(
+        verdict_of(&control_image, "I-9.6"),
+        InvariantVerdict::Holds,
+        "开关关着的同一段流 I-9.6 成立"
+    );
+    assert_eq!(
+        violated_invariants(&control_image),
+        Vec::<&str>::new(),
+        "开关关着的同一段流一条违例都没有"
+    );
+}
````

### s：mutations-append.tsv

````
C479：写入口装 inode 号水位开关那一句不生效（压回去也照记账的水位发号）	crates/singlefs-core/src/transaction.rs	        self.inode_number_watermark_source = source;	        let _ = source;	-p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number	creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant
C479：建 inode 的起号不读水位开关（压回去也从上一版记账的水位发）	crates/singlefs-core/src/transaction.rs	        .watermark_this_publish_starts_from(first_new_inode_number);	        .watermark_this_publish_starts_from(first_new_inode_number)\n        .max(first_new_inode_number);	-p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number	creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant
C479：记账里写的 inode 号水位不读水位开关（压回去也照上一版的水位加新建个数）	crates/singlefs-core/src/transaction.rs	            .watermark_this_publish_starts_from(previous_version.inode_number_watermark()),	            .watermark_this_publish_starts_from(previous_version.inode_number_watermark())\n            .max(previous_version.inode_number_watermark()),	-p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number	creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant
C479：水位开关压回去那一臂交回上一版的水位	crates/singlefs-core/src/transaction.rs	                );\n                pressed_back.next_inode_number()\n            }	                );\n                inode_number_watermark_of_the_previous_version\n            }	-p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number	creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant
C479：压回去的水位收无效 inode 号 0	crates/singlefs-core/src/transaction.rs	        (next_inode_number >= FIRST_INODE_NUMBER).then_some(Self(next_inode_number))	        Some(Self(next_inode_number))	-p singlefs-core --lib -- pressed_back_inode_number_watermark	the_pressed_back_inode_number_watermark_refuses_the_invalid_inode_number_zero
C479：水位开关压的号不小于上一版的水位也不拦	crates/singlefs-core/src/transaction.rs	                        < inode_number_watermark_of_the_previous_version,	                        < u64::MAX,	-p singlefs-core --lib -- pressed_back_inode_number_watermark	a_pressed_back_inode_number_watermark_not_below_the_previous_watermark_is_a_misused_switch
````

