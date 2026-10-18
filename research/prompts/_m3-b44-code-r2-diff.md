# m3-b44-code-r2 被判的改动（2026-10-17）

`crates/singlefs-core/src/` 下 `mount.rs`、`recovery.rs`、`transaction.rs`、`mounted_session.rs` 四份：W8 之前（主工作区此刻的这四份，在草稿里依次倒着撤掉 W11、W10、W8 三份补丁里这四份的那一段还原）与此刻的 `diff -u`，4 个文件 +641/−246。

```diff
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -30,7 +30,7 @@
 };
 use crate::journal::back_chain_of;
 use crate::make_filesystem::MakeFilesystemParameters;
-use crate::mounted_session::refusal_is_short_of_space;
+use crate::mounted_session::{refusal_is_short_of_space, DeviceKeepingWritesInMemory};
 use crate::pointer::LocationEntry;
 use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
 use crate::records::{
@@ -77,7 +77,8 @@
     prepare_the_version_publish, publish_instance_table_on_version_without_file, publish_version,
     publish_version_appending_a_lineage_record, publish_version_with_unmount_marker,
     publish_without_units, refuse_locations_that_do_not_name_two_pool_devices,
-    role_of_allocation_record_tree_node, walk_the_deletion_queue_until_it_is_empty,
+    rehearse_every_step_of_the_deletion_queue_walk, role_of_allocation_record_tree_node,
+    walk_the_deletion_queue_until_it_is_empty,
     write_the_raised_floor_into_every_system_configuration, AcquisitionFailed,
     CodeTwoTreeNodeCapacities, CopyQuarantinedAfterReleaseChecksumMismatch,
     DeletionAndLineageOfThePublish, DeletionQueueWalk, ExpectedInstanceAcquisitionFailed,
@@ -114,6 +115,8 @@
     Acquisition(AcquisitionFailed),
     /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成，或挂载续做里的一步没发成、不是空间不够：错，连同这次挂载交得出的写账
     /// （[`PublishSequenceFailed`]：续做那一格的已落盘各次是写行、暖机、推的抬 F 与续做里已落盘的每一次，按先后）。
+    /// 续做里有一步要开的容器号到顶（C841）在取号之前预演出来时同样交这个成员，那时一个写都没发，三份写账都是空的
+    /// （`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`）。
     /// 装箱（与 `RaiseFloorSequencePublishFailed` 同）：写账带上抬 F 先写系统配置那一步之后，不装箱 `MountError` 就大过 clippy
     /// `result_large_err` 的 128 字节。
     Publish(Box<PublishSequenceFailed>),
@@ -5099,7 +5102,7 @@
     rows_to_write: usize,
     warm_up_publish_txgs: &[CheckpointTxg],
     instance_to_acquire: InstanceGeneration,
-) -> Result<Vec<PlacementsTakenByOnePublish>, DryRunRefusal> {
+) -> Result<PublishesRehearsedBeforeAcquisition, DryRunRefusal> {
     let mut copy = allocator.clone();
     let refused_at = |publish_index: usize| {
         move |cause: PublishError| DryRunRefusal {
@@ -5215,7 +5218,121 @@
         taken_by_every_publish.push(placements_taken_by(&next));
         current = next;
     }
-    Ok(taken_by_every_publish)
+    Ok(PublishesRehearsedBeforeAcquisition {
+        placements_taken_by_every_publish: taken_by_every_publish,
+        version_after_the_last_publish: current,
+        allocator_after_the_last_publish: copy,
+    })
+}
+
+/// 取号之前整串预演（[`dry_run_of_the_publishes_after_acquisition`]）交回的：每一次取到的落点，预演到最后的那一版
+/// （写行与暖机都预演过之后的现行那一版：挂载续做在真发时从写行、暖机之后的那一版接着走），与预演到最后的那一份分配器拷贝
+/// （取号之前预演整个续做接着它取落点）。
+struct PublishesRehearsedBeforeAcquisition {
+    placements_taken_by_every_publish: Vec<PlacementsTakenByOnePublish>,
+    version_after_the_last_publish: PoolVersion,
+    allocator_after_the_last_publish: PoolAllocator,
+}
+
+/// 挂载续做（[`continue_the_deletion_queue_until_it_is_empty`]）在取号之前整个预演一遍（岔路单 `research/prompts/m3-b44-code-r1-forks.md`
+/// 那一行；代码轮 `research/prompts/m3-b44-code-r1-main-verification.md` 第二节 K6）：在取号之前那一串预演到最后的那一版与那一份分配器拷贝上，
+/// 照续做取的批大小与容器树容量（分配器上装的删除开关，续做读同一份）把删除队列走到空的每一步都只算不写
+/// （`transaction::rehearse_every_step_of_the_deletion_queue_walk`），真分配器不动。
+/// 那一趟有一步被拒是空间不够的那两种（[`refusal_is_short_of_space`]：续做真发时推一串抬 F 再试的就是这两种）时，抬 F 也预演进去：
+/// 在只进内存的盘与 `allocator` 的一份拷贝上把取号之后那一串连同续做照真发走一遍
+/// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]），取续做在那里报的错。
+/// 有一步报 C841（容器树要开新容器而最右那片的号已到顶）就在取号之前拒：不拒的话这一错在续做里才交回，那时写行那次发布与续做里
+/// 排在它前面的各步（连同推的抬 F）已落盘，之后每次可写挂载再取号、写行、再拒（攻方 `research/prompts/m3-b44-code-r1-opus-output.md` 第五节）。
+/// 预演报的别的错这里不拒、照旧留给续做（[`rehearsed_continuation_refusal_is_returned_before_acquisition`]）；分配器拷贝上那一趟报了
+/// 别的错（不是空间不够）的那一步之后接不上一版，不再预演。
+///
+/// # Errors
+/// C841 ⇒ `MountError::Publish`：`cause` 是那个错，三份写账都是空的（取号之前，一个写都没发到盘上）。
+fn refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range<
+    Device: BlockDevice,
+>(
+    pool: &PoolWriter<'_, Device>,
+    version_after_the_publishes_rehearsed: &PoolVersion,
+    mut allocator_after_the_publishes_rehearsed: PoolAllocator,
+    allocator: &PoolAllocator,
+    start: &InstanceStart,
+    rows_written: &[InstanceRow],
+    publishes_planned: &PublishesPlannedAfterAcquisition,
+) -> Result<(), MountError> {
+    let version_of_trees = match version_after_the_publishes_rehearsed {
+        PoolVersion::WithoutFile(_) => return Ok(()),
+        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
+    };
+    let deletion_switches_the_continuation_reads = allocator.deletion_switches();
+    let refusal = match rehearse_every_step_of_the_deletion_queue_walk(
+        pool,
+        &mut allocator_after_the_publishes_rehearsed,
+        version_of_trees,
+        deletion_switches_the_continuation_reads.batch_size,
+        deletion_switches_the_continuation_reads.container_tree_capacities(),
+        publishes_planned.instance_to_acquire,
+    ) {
+        Ok(()) => return Ok(()),
+        Err(refusal) if refusal_is_short_of_space(&refusal) => {
+            match walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory(
+                pool.parameters,
+                &*pool.devices,
+                allocator,
+                start,
+                rows_written,
+                publishes_planned,
+            ) {
+                Some(refusal_of_the_continuation) => refusal_of_the_continuation,
+                None => return Ok(()),
+            }
+        }
+        Err(refusal) => refusal,
+    };
+    if !rehearsed_continuation_refusal_is_returned_before_acquisition(&refusal) {
+        return Ok(());
+    }
+    Err(MountError::Publish(Box::new(PublishSequenceFailed {
+        cause: refusal,
+        writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
+        writes_of_persisted_publishes: Vec::new(),
+        writes_of_failed_publishes: Vec::new(),
+    })))
+}
+
+/// 取号之前预演整个续做报的错里，哪几种在取号之前拒（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
+/// 只有 C841 那一种（岔路单那一行只定了这一错）；别的照旧留给续做，续做里怎么处置见 [`continue_the_deletion_queue_until_it_is_empty`]。
+fn rehearsed_continuation_refusal_is_returned_before_acquisition(refusal: &PublishError) -> bool {
+    match refusal {
+        PublishError::NextContainerNumberPastTheTopOfItsRange(_) => true,
+        PublishError::SpaceAdmissionRefused(_)
+        | PublishError::PlacementRefused { .. }
+        | PublishError::MultiLevelCodeTwoTreeRefused { .. }
+        | PublishError::ReleaseNotInMapping { .. }
+        | PublishError::ReleaseTargetNotAllocated { .. }
+        | PublishError::ReleaseTargetAlreadyReleased { .. }
+        | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
+        | PublishError::ReleaseSpanMismatch { .. }
+        | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
+        | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
+        | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
+        | PublishError::ContentExceedsDataUnit { .. }
+        | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
+        | PublishError::InodeTreeWriteRefused(_)
+        | PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. }
+        | PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. }
+        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
+        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
+        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
+        | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
+        // txg 到顶不拒（岔路单那一行只定了 C841，别的错不扩大）：分配器拷贝上那一趟不算写行之后为空间推的抬 F，
+        // 续做各步的 txg 在那里算的不是真发的那一个。
+        | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
+        | PublishError::WriteToAnInodeBeingDeleted { .. }
+        | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
+        | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
+        | PublishError::DeletedInodesQueueRefused(_)
+        | PublishError::BlockDevice(_) => false,
+    }
 }
 
 /// 带文件的一版的一次发布真取到的落点，按取落点的次序：这次重写的每个角色（`TransactionOutput::rewritten`）。
@@ -6070,7 +6187,7 @@
         devices,
         unit_area_start_of_the_chosen_system_configuration(&start.system_configuration),
     )?;
-    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
+    let pool = PoolWriter::new(parameters, devices.as_mut_slice());
     let all_devices: Vec<DeviceIdentity> =
         pool.devices.iter().map(|(identity, _)| *identity).collect();
     // 暖机推哪几次，取号之前先算出来：准入按这一份算，取号之后的循环也按这一份推。
@@ -6140,7 +6257,11 @@
     // 取号之后要发的那一串（写行一次、暖机那几次）在取号之前整串预演一遍（分配器的拷贝上，走发布路径落盘之前那一段，
     // 与后面真发读同一个分配器——取号不碰它）：释放核验、树的形状、落点取不到，都在任何写之前返回。
     // 落点取不到怎么算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
-    let placements_planned = match dry_run_of_the_publishes_after_acquisition(
+    let PublishesRehearsedBeforeAcquisition {
+        placements_taken_by_every_publish,
+        version_after_the_last_publish,
+        allocator_after_the_last_publish,
+    } = match dry_run_of_the_publishes_after_acquisition(
         &pool,
         &allocator,
         &start,
@@ -6149,7 +6270,7 @@
         &warm_up_publishes_planned,
         instance_to_acquire,
     ) {
-        Ok(taken) => Some(taken),
+        Ok(rehearsed) => rehearsed,
         Err(DryRunRefusal {
             publish_index,
             cause: PublishError::PlacementRefused { unit, refusal },
@@ -6185,6 +6306,26 @@
             })
         }
     };
+    let placements_planned = Some(placements_taken_by_every_publish);
+    // 取号之后那一串（取号、写行、暖机、写行之后推的抬 F）要的、取号之前定下来的那几样：真发与取号之前预演续做读同一份。
+    let publishes_planned = PublishesPlannedAfterAcquisition {
+        instance_to_acquire,
+        instance_table_rewrite,
+        warm_up_publishes_planned,
+        placements_planned,
+        admission_before_acquisition,
+    };
+    // 挂载续做接着写行与暖机预演到最后的那一版整个预演一遍（续做要推抬 F 时连同取号之后那一串在只进内存的盘上走）：
+    // 有一步要开的容器号到顶（C841）就在取号之前拒，盘上逐字节不变；不预演的话这一错在写行那次发布落盘之后才在续做里交回。
+    refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range(
+        &pool,
+        &version_after_the_last_publish,
+        allocator_after_the_last_publish,
+        &allocator,
+        &start,
+        &rows_written,
+        &publishes_planned,
+    )?;
     // 可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13、D18（块里携带什么信息） 已定项 11）：取号是这次挂载的第一个写，
     // 核在它之前、读的是紧接着要写的这批盘；有一块不带所选那一版就不取号，盘上逐字节不变。排在别的准入之后，
     // 那几条先判出来的仍报各自的成员。落后支缺根那一半只核写进了根环的所选那一版：择到的那条根上没施加 journal 记录才是它
@@ -6225,211 +6366,21 @@
             },
         );
     }
-    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(
-        |failure| match failure {
-            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
-                expected,
-                recomputed,
-            } => MountError::InstanceGenerationChangedBeforeAcquisition {
-                expected,
-                recomputed,
-            },
-            ExpectedInstanceAcquisitionFailed::InstanceGenerationPastTheTopOfItsRange(
-                InstanceGenerationPastTheTopOfItsRange { highest_on_disk },
-            ) => MountError::SequenceNumberPastTheTopOfItsRange(
-                SequenceNumberAtTheTopOfItsRange::InstanceGeneration(highest_on_disk),
-            ),
-            // 读阶段的见证读判得出、取号那一刻读见证值时判不出（R2）或这块盘一份自证过的系统配置都没有（两次读之间的瞬时读错，
-            // C554 乙-配置续 Q1）：拒可写，与读阶段判据 N-配置 那一拒分开报（C664）。一个字节都没写。
-            ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
-                first_device_reporting_a_read_error,
-            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
-                start.rereads.read_stage,
-                SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
-                    first_device_reporting_a_read_error,
-                },
-            ),
-            ExpectedInstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
-                device,
-            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
-                start.rereads.read_stage,
-                SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
-                    device,
-                },
-            ),
-            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {
-                MountError::Acquisition(acquisition)
-            }
-        },
+    let PublishedBetweenAcquisitionAndTheContinuation {
+        instance,
+        row_publish,
+        warm_up_publishes,
+        space_admission,
+        mut current,
+        mut allocator,
+    } = acquire_then_publish_until_the_continuation(
+        parameters,
+        devices,
+        allocator,
+        &start,
+        &rows_written,
+        publishes_planned,
     )?;
-    assert_eq!(
-        instance, instance_to_acquire,
-        "acquire_expected_instance 写之前重算、与判定时的号不等就不写：交回的就是列行与判准入用的那个号"
-    );
-
-    // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishSequenceFailed`）：写入口随错一起丢掉。
-    let row_publish = match &start.previous {
-        PreviousVersion::WithFile { output, .. }
-        | PreviousVersion::WithTreesAndNoFile { output, .. } => {
-            let published = publish_rows_on_file_version(
-                &mut pool,
-                &mut allocator,
-                output,
-                instance_table_rewrite,
-                RowPublishIdentity {
-                    txg: start.first_txg,
-                    counter: start.next_counter,
-                    instance,
-                    rollback_floor: start.effective_floor,
-                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
-                },
-            );
-            published.map(PoolVersion::of_a_version_of_trees)
-        }
-        // 树表 0 条的一版：要写的行为空（上一个实例是 0，第一次可写挂载）就走零单元发布，新池新建文件的字节不变
-        // （D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」；layout/01-first-txn.md 八「第一次之后的可写挂载（写行）」那一行 2026-09-14 的改名注）；
-        // 要写的行不为空就只重写实例表那一个单元——这一版没有记账树，已定项 9 的那五样一样都不写，而 D18（块里携带什么信息） 已定项 11
-        // 要求每次可写挂载都写行。
-        PreviousVersion::WithoutFile { root, .. } if rows_written.is_empty() => {
-            publish_without_units(
-                &mut pool,
-                root,
-                ZeroUnitPublishPlan {
-                    txg: start.first_txg,
-                    counter: start.next_counter,
-                    instance,
-                    back_chain: 0,
-                    rollback_floor: start.effective_floor,
-                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
-                },
-            )
-            .map(|published| {
-                // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`），
-                // 记之前照 E2 重读挂载时读不出的那一槽。
-                record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
-                    &*pool.devices,
-                    pool.parameters,
-                    &mut allocator,
-                    published.root.checkpoint_txg,
-                );
-                PoolVersion::WithoutFile(published)
-            })
-        }
-        PreviousVersion::WithoutFile { root, .. } => {
-            publish_instance_table_on_version_without_file(
-                &mut pool,
-                &mut allocator,
-                root,
-                InstanceTableOnlyPublishPlan {
-                    txg: start.first_txg,
-                    counter: start.next_counter,
-                    instance,
-                    // 新实例的第一条记录，反向链恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
-                    back_chain: 0,
-                    rollback_floor: start.effective_floor,
-                    instance_table: &instance_table_rewrite,
-                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
-                },
-            )
-            .map(PoolVersion::WithoutFile)
-        }
-    }
-    .map_err(|cause| publish_failed_after_acquisition(&pool, &[], cause))?;
-
-    // 暖机（D16（发布语义） 已定项 8 戊）：本实例的根覆盖两块盘之前连推空发布，推哪几次按取号之前算好的那一份计划
-    // （`warm_up_publishes_planned`），不在这里另算一遍——两处各算各的，只要有一处多算一次，取号之前那道准入就罩不住实际推的那几次。
-    assert_eq!(
-        row_publish.root().checkpoint_txg,
-        start.first_txg,
-        "写行那次发布的 txg 就是算暖机计划用的那个：两条路（带文件的写行、树表 0 条的零单元发布）交给发布路径的 txg 都取 start.first_txg"
-    );
-    let mut current = row_publish.clone();
-    let mut warm_up_publishes = Vec::new();
-    for planned_txg in &warm_up_publishes_planned {
-        assert_eq!(
-            checkpoint_txg_of_the_planned_publish_after(current.root().checkpoint_txg),
-            *planned_txg,
-            "这次空发布要用的 txg 与计划里的那一个相同：`publish_empty_after` 取的是现行那一版的 txg 加一，计划也是逐个加一"
-        );
-        let next = publish_empty_after(&mut pool, &mut allocator, &current, instance).map_err(
-            |cause| {
-                let persisted: Vec<&PoolVersion> = std::iter::once(&row_publish)
-                    .chain(&warm_up_publishes)
-                    .collect();
-                publish_failed_after_acquisition(&pool, &persisted, cause)
-            },
-        )?;
-        warm_up_publishes.push(next.clone());
-        current = next;
-    }
-    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开
-    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。
-    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后
-    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；
-    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，
-    // 扣住的那一截不交给任何会话。
-    allocator.release_reclaim_holds();
-    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——
-    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同
-    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
-    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)
-        .chain(&warm_up_publishes)
-        .map(placements_taken_by)
-        .collect();
-    let some_copy_was_quarantined = std::iter::once(&row_publish)
-        .chain(&warm_up_publishes)
-        .filter_map(PoolVersion::version_of_trees)
-        .any(|version| {
-            !version
-                .quarantined_after_release_checksum_mismatch
-                .is_empty()
-        });
-    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {
-        assert_eq!(
-            &placements_taken, planned,
-            "取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
-             不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串"
-        );
-    }
-
-    let space_admission = match admission_before_acquisition {
-        WritableMountAdmissionBeforeAcquisition::Admitted => {
-            MountSpaceAdmission::AdmittedBeforeAcquisition
-        }
-        WritableMountAdmissionBeforeAcquisition::NotJudgedByTheTestOnlySwitch => {
-            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch
-        }
-        WritableMountAdmissionBeforeAcquisition::ShortUntilTheFloorIsRaisedAfterTheRowPublish(
-            refusal_before_acquisition,
-        ) => {
-            let current_file_version = match &mut current {
-                PoolVersion::WithFile(current_file_version)
-                | PoolVersion::WithTreesAndNoFile(current_file_version) => current_file_version,
-                PoolVersion::WithoutFile(_) => panic!(
-                    "取号之前不够、要写行之后推的只有上一版有树的那一格（树表 0 条的在取号之前拒了）：\
-                     有树的一版上写行与暖机交回的都是有树的一版（`publish_rows_on_file_version`、`publish_empty_after`）"
-                ),
-            };
-            // 这次挂载已落盘的发布（写行那次、暖机每次）各自的写：抬 F 报错时随错交出（实审 A1b Q5）。
-            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
-                std::iter::once(&row_publish)
-                    .chain(&warm_up_publishes)
-                    .map(|version| version.writes().clone())
-                    .collect();
-            // 写行与暖机那几次的写入口装的节点容量：之后推抬 F、再判的读数按同一档算。
-            let node_capacities_of_the_writer = pool.code_two_tree_node_capacities();
-            push_floor_raises_after_the_row_publish(
-                parameters,
-                devices,
-                &mut allocator,
-                current_file_version,
-                start.shadow_ledger,
-                node_capacities_of_the_writer,
-                refusal_before_acquisition,
-                writes_of_persisted_publishes,
-            )?
-        }
-    };
 
     // 续做之前这次挂载已落盘的发布（写行那次、暖机每次、写行之后推的抬 F 每一次空发布）各自的写，按先后：续做里的一步没发成、
     // 不是空间不够时随错交出（`PublishSequenceFailed::writes_of_persisted_publishes` 的契约，代码轮 U9）。
@@ -6533,7 +6484,9 @@
 }
 
 /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）：写行与暖机（连同之后为空间推的抬 F）之后、交回之前，把现行那一版的 `deleted_inodes`
-/// 走到空；与会话每次用户改动先走空共用同一个走空函数（`transaction::walk_the_deletion_queue_until_it_is_empty`），不进取号之前的预演。
+/// 走到空；与会话每次用户改动先走空共用同一个走空函数（`transaction::walk_the_deletion_queue_until_it_is_empty`）。取号之前把它的每一步
+/// （为空间推的抬 F 在内）预演一遍、只拒 C841
+/// （[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）。
 /// 批大小与墓碑容器容量取分配器上装的开关（`mount_writable_with_test_only_switches_and_deletion_switches`）。队列空（常态）时一个写都不发；
 /// 走完 `current` 按 `PoolVersion::of_a_version_of_trees` 重新归成员。交回发成的每一步（按先后）、为空间推的那几串抬 F 与停在哪儿。
 ///
@@ -6717,7 +6670,334 @@
     added_up
 }
 
+/// 取号之后、续做之前那一串要的、取号之前定下来的那几样（`establish_instance` 算出来）：要取的号、写行那次重写的实例表、
+/// 暖机推哪几次、取号之前在分配器拷贝上预演取到的落点（真发之后拿它比）、取号之前那一判的结局。
+/// 真发（[`acquire_then_publish_until_the_continuation`]）与取号之前在只进内存的盘上预演续做
+/// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]）读同一份。
+#[derive(Clone)]
+struct PublishesPlannedAfterAcquisition {
+    instance_to_acquire: InstanceGeneration,
+    instance_table_rewrite: InstanceTableRewrite,
+    warm_up_publishes_planned: Vec<CheckpointTxg>,
+    placements_planned: Option<Vec<PlacementsTakenByOnePublish>>,
+    admission_before_acquisition: WritableMountAdmissionBeforeAcquisition,
+}
+
+/// 取号之后、续做之前那一串发成的（[`acquire_then_publish_until_the_continuation`]）：取到的号、写行那次、暖机每次、
+/// 写行之后那一判（连同推的抬 F）、现行那一版与发完之后的分配器。
+struct PublishedBetweenAcquisitionAndTheContinuation {
+    instance: InstanceGeneration,
+    row_publish: PoolVersion,
+    warm_up_publishes: Vec<PoolVersion>,
+    space_admission: MountSpaceAdmission,
+    current: PoolVersion,
+    allocator: PoolAllocator,
+}
+
+/// 取号、写行、暖机，取号之前不够的再推抬 F（`push_floor_raises_after_the_row_publish`）：可写挂载取号之后、续做之前那一串。
+/// 可写挂载真发走它（`establish_instance`）；取号之前预演续做要推抬 F 时，在只进内存的盘与分配器的一份拷贝上也走它
+/// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]），两边是同一段代码。
+/// 写入口在这里按 `devices` 开，与取号之前预演那一串的写入口同一份参数、同一档节点容量（都是 `PoolWriter::new` 的那一档）。
+///
+/// # Errors
+/// 取号的错（号变了、到顶、系统配置读不出、取号自己的错）；写行与暖机发不出去（`MountError::Publish`，带这次挂载的写账）；
+/// 写行之后推抬 F 时抬 F 自己报的错（[`push_floor_raises_after_the_row_publish`]）。
+fn acquire_then_publish_until_the_continuation<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    mut allocator: PoolAllocator,
+    start: &InstanceStart,
+    rows_written: &[InstanceRow],
+    publishes_planned: PublishesPlannedAfterAcquisition,
+) -> Result<PublishedBetweenAcquisitionAndTheContinuation, MountError> {
+    let PublishesPlannedAfterAcquisition {
+        instance_to_acquire,
+        instance_table_rewrite,
+        warm_up_publishes_planned,
+        placements_planned,
+        admission_before_acquisition,
+    } = publishes_planned;
+    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
+    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(
+        |failure| match failure {
+            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
+                expected,
+                recomputed,
+            } => MountError::InstanceGenerationChangedBeforeAcquisition {
+                expected,
+                recomputed,
+            },
+            ExpectedInstanceAcquisitionFailed::InstanceGenerationPastTheTopOfItsRange(
+                InstanceGenerationPastTheTopOfItsRange { highest_on_disk },
+            ) => MountError::SequenceNumberPastTheTopOfItsRange(
+                SequenceNumberAtTheTopOfItsRange::InstanceGeneration(highest_on_disk),
+            ),
+            // 读阶段的见证读判得出、取号那一刻读见证值时判不出（R2）或这块盘一份自证过的系统配置都没有（两次读之间的瞬时读错，
+            // C554 乙-配置续 Q1）：拒可写，与读阶段判据 N-配置 那一拒分开报（C664）。一个字节都没写。
+            ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+                first_device_reporting_a_read_error,
+            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
+                start.rereads.read_stage,
+                SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
+                    first_device_reporting_a_read_error,
+                },
+            ),
+            ExpectedInstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
+                device,
+            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
+                start.rereads.read_stage,
+                SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
+                    device,
+                },
+            ),
+            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {
+                MountError::Acquisition(acquisition)
+            }
+        },
+    )?;
+    assert_eq!(
+        instance, instance_to_acquire,
+        "acquire_expected_instance 写之前重算、与判定时的号不等就不写：交回的就是列行与判准入用的那个号"
+    );
+
+    // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishSequenceFailed`）：写入口随错一起丢掉。
+    let row_publish = match &start.previous {
+        PreviousVersion::WithFile { output, .. }
+        | PreviousVersion::WithTreesAndNoFile { output, .. } => {
+            let published = publish_rows_on_file_version(
+                &mut pool,
+                &mut allocator,
+                output,
+                instance_table_rewrite,
+                RowPublishIdentity {
+                    txg: start.first_txg,
+                    counter: start.next_counter,
+                    instance,
+                    rollback_floor: start.effective_floor,
+                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
+                },
+            );
+            published.map(PoolVersion::of_a_version_of_trees)
+        }
+        // 树表 0 条的一版：要写的行为空（上一个实例是 0，第一次可写挂载）就走零单元发布，新池新建文件的字节不变
+        // （D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」；layout/01-first-txn.md 八「第一次之后的可写挂载（写行）」那一行 2026-09-14 的改名注）；
+        // 要写的行不为空就只重写实例表那一个单元——这一版没有记账树，已定项 9 的那五样一样都不写，而 D18（块里携带什么信息） 已定项 11
+        // 要求每次可写挂载都写行。
+        PreviousVersion::WithoutFile { root, .. } if rows_written.is_empty() => {
+            publish_without_units(
+                &mut pool,
+                root,
+                ZeroUnitPublishPlan {
+                    txg: start.first_txg,
+                    counter: start.next_counter,
+                    instance,
+                    back_chain: 0,
+                    rollback_floor: start.effective_floor,
+                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
+                },
+            )
+            .map(|published| {
+                // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`），
+                // 记之前照 E2 重读挂载时读不出的那一槽。
+                record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
+                    &*pool.devices,
+                    pool.parameters,
+                    &mut allocator,
+                    published.root.checkpoint_txg,
+                );
+                PoolVersion::WithoutFile(published)
+            })
+        }
+        PreviousVersion::WithoutFile { root, .. } => {
+            publish_instance_table_on_version_without_file(
+                &mut pool,
+                &mut allocator,
+                root,
+                InstanceTableOnlyPublishPlan {
+                    txg: start.first_txg,
+                    counter: start.next_counter,
+                    instance,
+                    // 新实例的第一条记录，反向链恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
+                    back_chain: 0,
+                    rollback_floor: start.effective_floor,
+                    instance_table: &instance_table_rewrite,
+                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
+                },
+            )
+            .map(PoolVersion::WithoutFile)
+        }
+    }
+    .map_err(|cause| publish_failed_after_acquisition(&pool, &[], cause))?;
+
+    // 暖机（D16（发布语义） 已定项 8 戊）：本实例的根覆盖两块盘之前连推空发布，推哪几次按取号之前算好的那一份计划
+    // （`warm_up_publishes_planned`），不在这里另算一遍——两处各算各的，只要有一处多算一次，取号之前那道准入就罩不住实际推的那几次。
+    assert_eq!(
+        row_publish.root().checkpoint_txg,
+        start.first_txg,
+        "写行那次发布的 txg 就是算暖机计划用的那个：两条路（带文件的写行、树表 0 条的零单元发布）交给发布路径的 txg 都取 start.first_txg"
+    );
+    let mut current = row_publish.clone();
+    let mut warm_up_publishes = Vec::new();
+    for planned_txg in &warm_up_publishes_planned {
+        assert_eq!(
+            checkpoint_txg_of_the_planned_publish_after(current.root().checkpoint_txg),
+            *planned_txg,
+            "这次空发布要用的 txg 与计划里的那一个相同：`publish_empty_after` 取的是现行那一版的 txg 加一，计划也是逐个加一"
+        );
+        let next = publish_empty_after(&mut pool, &mut allocator, &current, instance).map_err(
+            |cause| {
+                let persisted: Vec<&PoolVersion> = std::iter::once(&row_publish)
+                    .chain(&warm_up_publishes)
+                    .collect();
+                publish_failed_after_acquisition(&pool, &persisted, cause)
+            },
+        )?;
+        warm_up_publishes.push(next.clone());
+        current = next;
+    }
+    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开
+    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。
+    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后
+    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；
+    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，
+    // 扣住的那一截不交给任何会话。
+    allocator.release_reclaim_holds();
+    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——
+    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同
+    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
+    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)
+        .chain(&warm_up_publishes)
+        .map(placements_taken_by)
+        .collect();
+    let some_copy_was_quarantined = std::iter::once(&row_publish)
+        .chain(&warm_up_publishes)
+        .filter_map(PoolVersion::version_of_trees)
+        .any(|version| {
+            !version
+                .quarantined_after_release_checksum_mismatch
+                .is_empty()
+        });
+    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {
+        assert_eq!(
+            &placements_taken, planned,
+            "取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
+             不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串"
+        );
+    }
+
+    let space_admission = match admission_before_acquisition {
+        WritableMountAdmissionBeforeAcquisition::Admitted => {
+            MountSpaceAdmission::AdmittedBeforeAcquisition
+        }
+        WritableMountAdmissionBeforeAcquisition::NotJudgedByTheTestOnlySwitch => {
+            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch
+        }
+        WritableMountAdmissionBeforeAcquisition::ShortUntilTheFloorIsRaisedAfterTheRowPublish(
+            refusal_before_acquisition,
+        ) => {
+            let current_file_version = match &mut current {
+                PoolVersion::WithFile(current_file_version)
+                | PoolVersion::WithTreesAndNoFile(current_file_version) => current_file_version,
+                PoolVersion::WithoutFile(_) => panic!(
+                    "取号之前不够、要写行之后推的只有上一版有树的那一格（树表 0 条的在取号之前拒了）：\
+                     有树的一版上写行与暖机交回的都是有树的一版（`publish_rows_on_file_version`、`publish_empty_after`）"
+                ),
+            };
+            // 这次挂载已落盘的发布（写行那次、暖机每次）各自的写：抬 F 报错时随错交出（实审 A1b Q5）。
+            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
+                std::iter::once(&row_publish)
+                    .chain(&warm_up_publishes)
+                    .map(|version| version.writes().clone())
+                    .collect();
+            // 写行与暖机那几次的写入口装的节点容量：之后推抬 F、再判的读数按同一档算。
+            let node_capacities_of_the_writer = pool.code_two_tree_node_capacities();
+            push_floor_raises_after_the_row_publish(
+                parameters,
+                devices,
+                &mut allocator,
+                current_file_version,
+                start.shadow_ledger,
+                node_capacities_of_the_writer,
+                refusal_before_acquisition,
+                writes_of_persisted_publishes,
+            )?
+        }
+    };
+    Ok(PublishedBetweenAcquisitionAndTheContinuation {
+        instance,
+        row_publish,
+        warm_up_publishes,
+        space_admission,
+        current,
+        allocator,
+    })
+}
+
+/// 取号之前预演续做、续做要推抬 F 的那一格（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
+/// 分配器拷贝上那一趟（`transaction::rehearse_every_step_of_the_deletion_queue_walk`）不读不写盘，推不了抬 F——抬 F 要读根环、实例表与
+/// 被抛弃根的账，还要先写系统配置、再发空发布——空间不够的那一步之后就接不上一版。这里改在只进内存的盘上
+/// （`mounted_session::DeviceKeepingWritesInMemory`：读穿到底下那块盘，写只记进内存）与 `allocator` 的一份拷贝上，把取号之后那一串
+/// （[`acquire_then_publish_until_the_continuation`]）连同续做（[`continue_the_deletion_queue_until_it_is_empty`]：被拒是空间不够就推一串抬 F、
+/// 再试，推满仍放不下就停）照真发走一遍：同一份计划、同一个推抬 F 的判法与同一段抬 F，续做报的错就是真发时会报的那一个。
+/// 底下那几块盘一个写都不发，真分配器不动。
+///
+/// 交回续做里那一步报的错（`MountError::Publish` 的 `cause`：不是空间不够的那一种）；续做走到空、推满仍放不下、抬 F 自己报错，
+/// 或取号之后那一串在续做之前就报错，交回 `None`：那些照旧留给真发，真发时报它自己的。
+fn walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory<
+    Device: BlockDevice,
+>(
+    parameters: &MakeFilesystemParameters,
+    devices: &[(DeviceIdentity, Device)],
+    allocator: &PoolAllocator,
+    start: &InstanceStart,
+    rows_written: &[InstanceRow],
+    publishes_planned: &PublishesPlannedAfterAcquisition,
+) -> Option<PublishError> {
+    let mut devices_keeping_writes_in_memory: Vec<(
+        DeviceIdentity,
+        DeviceKeepingWritesInMemory<'_>,
+    )> = devices
+        .iter()
+        .map(|(identity, device)| {
+            let disk: &dyn BlockDevice = device;
+            (*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))
+        })
+        .collect();
+    let PublishedBetweenAcquisitionAndTheContinuation {
+        instance,
+        space_admission,
+        mut current,
+        mut allocator,
+        ..
+    } = acquire_then_publish_until_the_continuation(
+        parameters,
+        &mut devices_keeping_writes_in_memory,
+        allocator.clone(),
+        start,
+        rows_written,
+        publishes_planned.clone(),
+    )
+    .ok()?;
+    let continuation = continue_the_deletion_queue_until_it_is_empty(
+        parameters,
+        &mut devices_keeping_writes_in_memory,
+        &mut allocator,
+        &mut current,
+        instance,
+        start.shadow_ledger,
+        // 续做之前已落盘的写只随续做的错一起交出、不影响续做怎么走：这里只取那个错，不另算这一份。
+        Vec::new(),
+        space_admission.floor_raises(),
+    );
+    if let Err(MountError::Publish(failed)) = continuation {
+        Some(failed.cause)
+    } else {
+        None
+    }
+}
+
 /// 可写挂载取号之前那一判的结局（`establish_instance` 用，交回之前换成 [`MountSpaceAdmission`]）。
+#[derive(Clone)]
 enum WritableMountAdmissionBeforeAcquisition {
     Admitted,
     /// 不够，上一版带文件：写行与暖机照做，之后推抬 F 的空发布再判（`push_floor_raises_after_the_row_publish`）。
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -5115,13 +5115,49 @@
     Ok(())
 }
 
+/// 一棵容器树（墓碑树、谱系树、退役树）这一版里盘上读来的指针：每片容器的指针与根之下每个码 2 节点的指针（根的指针住树表条目，
+/// 随树表条目判）。下一次重写这棵树的某个节点时，没换的条目连同它们的指针照抄写回。
+fn container_tree_pointers_read_from_disk<'version>(
+    container_pointers: impl Iterator<Item = &'version NodePointer> + 'version,
+    index_nodes: &'version ContainerTreeIndexNodesVersion,
+) -> impl Iterator<Item = &'version NodePointer> + 'version {
+    container_pointers.chain(
+        index_nodes
+            .pointers_below_the_root
+            .iter()
+            .map(|(_, pointer)| pointer),
+    )
+}
+
 /// 重建出来的这一版里每一条盘上读来的指针按 I-2.5 判（C476 普查「缺口」那张表 `pointer.rs` 那一行，实审 A3a 实测坐实走得到）：
-/// 下一次发布照抄没重写的角色时把它们原样写回（树表条目、多层码 2 树与分配记录树的内部条目、inode 树根的条目、extent 树），
-/// 写者 `NodePointer::write_to` / `DataPointer::write_to` 在那里断言升序——不在读进来的地方判，坏镜像就在写者的断言上 panic。
+/// 下一次发布照抄没重写的角色时把它们原样写回（树表条目、多层码 2 树与分配记录树的内部条目、inode 树根的条目、extent 树、
+/// 三棵容器树的内部条目），写者 `NodePointer::write_to` / `DataPointer::write_to` 在那里断言升序——不在读进来的地方判，
+/// 坏镜像就在写者的断言上 panic（三棵容器树那一份：代码轮 `research/prompts/m3-b44-code-r1-main-verification.md` 第二节 K4）。
 fn pointers_of_the_rebuilt_version_ascend_by_device(
     output: &TransactionOutput,
 ) -> Result<(), RecoveryFailure> {
     root_record_pointers_ascend_by_device(&output.root)?;
+    let tombstone_tree_pointers = container_tree_pointers_read_from_disk(
+        output
+            .tombstone_containers
+            .iter()
+            .map(|container| &container.pointer),
+        &output.tombstone_tree_index_nodes,
+    );
+    let lineage_tree_pointers = container_tree_pointers_read_from_disk(
+        output
+            .lineage_containers
+            .iter()
+            .map(|container| &container.pointer),
+        &output.lineage_tree_index_nodes,
+    );
+    let retirement_tree_pointers = container_tree_pointers_read_from_disk(
+        output
+            .retirement_containers
+            .iter()
+            .map(|container| &container.pointer),
+        &output.retirement_tree_index_nodes,
+    );
     let node_pointers = output
         .tree_table_entries
         .iter()
@@ -5154,7 +5190,10 @@
                 .map(|(_, pointer)| pointer),
         )
         .chain(output.accounting_tree.pointers.iter())
-        .chain(output.central_mapping_tree.pointers.iter());
+        .chain(output.central_mapping_tree.pointers.iter())
+        .chain(tombstone_tree_pointers)
+        .chain(lineage_tree_pointers)
+        .chain(retirement_tree_pointers);
     for pointer in node_pointers {
         pointer
             .location_entries_ascend_by_device()
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -6182,6 +6182,33 @@
     batch_size: UnlinkBatchSize,
     instance: InstanceGeneration,
 ) -> Result<TransactionOutput, PublishError> {
+    let (txg, deletion_and_lineage) = next_unlink_step_after(
+        previous,
+        inode,
+        batch_size,
+        pool.container_tree_capacities(),
+    )?;
+    publish_deletion_step(
+        pool,
+        allocator,
+        previous,
+        txg,
+        &deletion_and_lineage,
+        instance,
+    )
+}
+
+/// `inode` 接在 `previous` 后面的下一步：这一步的 txg 与交给发布路径的删除那一半。[`publish_unlink_step`] 真发与可写挂载取号之前
+/// 预演整个续做（[`rehearse_every_step_of_the_deletion_queue_walk`]）共用这一份，两处算的是同一个计划。
+///
+/// # Errors
+/// 同 [`publish_unlink_step`] 的头两样：`UnlinkOfAnInodeThatIsNotALiveRecord`、`NextCheckpointTxgPastTheTopOfItsRange`。
+fn next_unlink_step_after(
+    previous: &TransactionOutput,
+    inode: InodeNumber,
+    batch_size: UnlinkBatchSize,
+    container_tree_capacities: ContainerTreeCapacities,
+) -> Result<(CheckpointTxg, DeletionAndLineageOfThePublish), PublishError> {
     let record = previous
         .inode_leaf_containers
         .iter()
@@ -6218,19 +6245,58 @@
         },
         UnlinkStep::FinalBatch { .. } => DeletionStepOfThePublish::FinalBatch { inode },
     };
-    let deletion_and_lineage = DeletionAndLineageOfThePublish {
-        step,
-        container_tree_capacities: pool.container_tree_capacities(),
-        lineage: LineageStepOfThePublish::NoLineageRecord,
-    };
-    publish_deletion_step(
-        pool,
-        allocator,
-        previous,
+    Ok((
         txg,
-        &deletion_and_lineage,
-        instance,
-    )
+        DeletionAndLineageOfThePublish {
+            step,
+            container_tree_capacities,
+            lineage: LineageStepOfThePublish::NoLineageRecord,
+        },
+    ))
+}
+
+/// 把删除队列从 `current` 走到空的每一步都只算不写（可写挂载在取号之前预演整个挂载续做，岔路单
+/// `research/prompts/m3-b44-code-r1-forks.md` 那一行）：每一步与真发（[`walk_the_deletion_queue_until_it_is_empty`]）同一张计划
+/// （[`next_unlink_step_after`]、[`deletion_step_plan`]，树号取上一步那一版的），走发布路径落盘之前的那一段（[`prepare_the_version_publish`]，
+/// 不读盘核被换下的、照常释放），接在上一步预演出来的那一版上。落点取在 `allocator_copy` 上：调用方交的是分配器的拷贝，真分配器不动，
+/// 一个写都不发。队列空（常态）时什么都不算，交 `Ok`。
+///
+/// 迭代上界同 [`walk_the_deletion_queue_until_it_is_empty`]：每一步要么让队首那个文件少至少一个单元、要么删掉队首那个号。
+/// 跨轮携带的是预演到的那一版与 `allocator_copy`。
+///
+/// # Errors
+/// 走到的第一步报的错，原样（`NextContainerNumberPastTheTopOfItsRange` 在内）：那一步之后的各步接不上一版，不再预演。
+pub(crate) fn rehearse_every_step_of_the_deletion_queue_walk<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
+    allocator_copy: &mut PoolAllocator,
+    current: &TransactionOutput,
+    batch_size: UnlinkBatchSize,
+    container_tree_capacities: ContainerTreeCapacities,
+    instance: InstanceGeneration,
+) -> Result<(), PublishError> {
+    let mut version_rehearsed = current.clone();
+    loop {
+        let Some(inode) = version_rehearsed.deleted_inodes.first().copied() else {
+            return Ok(());
+        };
+        let (txg, deletion_and_lineage) = next_unlink_step_after(
+            &version_rehearsed,
+            inode,
+            batch_size,
+            container_tree_capacities,
+        )?;
+        let (_, version_after_the_rehearsed_step) = prepare_the_version_publish(
+            pool,
+            allocator_copy,
+            &deletion_step_plan(&version_rehearsed, txg, instance),
+            Some(&version_rehearsed),
+            version_rehearsed.tree_identifiers,
+            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
+            UnmountMarker::NotWrittenByTheUnmountSequence,
+            &deletion_and_lineage,
+        )?;
+        version_rehearsed = version_after_the_rehearsed_step;
+    }
 }
 
 /// 把现行那一版的 `deleted_inodes` 队列走空（D8（核心索引结构） 已定项 16 第 6 条：会话里每一次用户改动先走空、unlink 一次调用走完全部批，
@@ -6301,25 +6367,10 @@
     deletion_and_lineage: &DeletionAndLineageOfThePublish,
     instance: InstanceGeneration,
 ) -> Result<TransactionOutput, PublishError> {
-    let transaction_of_this_deletion = previous.highest_transaction_number_in_this_instance + 1;
     publish_version_of_trees(
         pool,
         allocator,
-        PublishPlan {
-            txg,
-            counter: previous.record.counter + 1,
-            transaction: transaction_of_this_deletion,
-            highest_transaction_number_before_this_publish: previous
-                .highest_transaction_number_in_this_instance,
-            instance,
-            back_chain: back_chain_of(&previous.record_bytes),
-            file: None,
-            new_inode_records: &[],
-            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
-            tree_birth_txg: previous.tree_birth_txg(),
-            tree_identifier_watermark: previous.root.tree_identifier_watermark,
-            rollback_floor: previous.root.rollback_floor,
-        },
+        deletion_step_plan(previous, txg, instance),
         Some(previous),
         previous.tree_identifiers,
         UnmountMarker::NotWrittenByTheUnmountSequence,
@@ -6327,6 +6378,30 @@
     )
 }
 
+/// 删除的三种发布接在 `previous` 后面那一次的计划（[`publish_deletion_step`] 真发与挂载取号之前预演整个续做共用）。
+fn deletion_step_plan(
+    previous: &TransactionOutput,
+    txg: CheckpointTxg,
+    instance: InstanceGeneration,
+) -> PublishPlan<'static> {
+    let transaction_of_this_deletion = previous.highest_transaction_number_in_this_instance + 1;
+    PublishPlan {
+        txg,
+        counter: previous.record.counter + 1,
+        transaction: transaction_of_this_deletion,
+        highest_transaction_number_before_this_publish: previous
+            .highest_transaction_number_in_this_instance,
+        instance,
+        back_chain: back_chain_of(&previous.record_bytes),
+        file: None,
+        new_inode_records: &[],
+        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
+        tree_birth_txg: previous.tree_birth_txg(),
+        tree_identifier_watermark: previous.root.tree_identifier_watermark,
+        rollback_floor: previous.root.rollback_floor,
+    }
+}
+
 /// 写入口拒删除中的 inode 与没有记录的 inode（D8（核心索引结构） 已定项 16 第 4 条）：覆盖写与顺序写现造第一个文件那条记录
 /// （`inode_record_of_file_version` 照 nlink = 1 写），上一版那条记录 nlink = 0、或已经删完不在树里时不许走到那一步；
 /// 活着的那条交回（覆盖写要接着它的对象出生代）。
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -354,10 +354,11 @@
 }
 
 /// 预演用的盘：读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘。
-/// 要出路时预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
+/// 要出路时预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）；可写挂载取号之前预演续做、
+/// 续做要推抬 F 时，取号之后那一串连同续做也在它上面走一遍（`mount` 的 `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`）。
 /// 底下那块盘只读借成 `&dyn BlockDevice`：改盘的三个动作要 `&mut`，类型上调不到。请求先过两个后端那一份判据
 /// （`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒。
-struct DeviceKeepingWritesInMemory<'disk> {
+pub(crate) struct DeviceKeepingWritesInMemory<'disk> {
     disk: &'disk dyn BlockDevice,
     physical_block_size: PhysicalBlockSizeInBytes,
     /// 写过的物理块：块号（按 `physical_block_size`）→ 那一块现在的字节。写零也逐块记成全 0（发布路径不写零，只有 mkfs 写零）。
@@ -365,7 +366,7 @@
 }
 
 impl<'disk> DeviceKeepingWritesInMemory<'disk> {
-    fn over_the_disk(disk: &'disk dyn BlockDevice) -> Self {
+    pub(crate) fn over_the_disk(disk: &'disk dyn BlockDevice) -> Self {
         DeviceKeepingWritesInMemory {
             disk,
             physical_block_size: disk.probe_physical_block_size(),
```
