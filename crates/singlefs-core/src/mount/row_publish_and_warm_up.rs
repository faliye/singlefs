//! 写行那次发布与暖机的空发布：两者的计划与发法、要写的行与暖机的 txg，一次发布取到的落点，
//! 一串发布里有一次失败时交回的账，所选那一版在 journal 里的位置。

use super::*;

/// 写行那次发布、上一版带文件的计划：在上一版那张实例表后面接上这次写的行，重写整条实例表链与固定点单元（D18（块里携带什么信息） 已定项 11；
/// 记账树已经存在 ⇒ 空发布也重写固定点单元，D16（发布语义） 已定项 9）。事务号 0、本实例第一条反向链 0。
/// 取号之后的发布与取号之前的预演读的是同一张计划（`publish_rows_on_file_version`、`dry_run_of_the_publishes_after_acquisition`）。
pub(super) fn row_publish_plan_on_file_version<'plan>(
    previous: &TransactionOutput,
    instance_table: InstanceTableRewrite,
    identity: &RowPublishIdentity,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: identity.txg,
        counter: identity.counter,
        transaction: 0,
        // 新实例的第一条记录（反向链恒 0），事务号按实例各算各的，从这里重新从 1 起。
        highest_transaction_number_before_this_publish: 0,
        instance: identity.instance,
        back_chain: 0,
        file: None,
        // 写行那次发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Rewrite(instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        tree_identifier_watermark: identity.tree_identifier_watermark,
        rollback_floor: identity.rollback_floor,
    }
}

/// 写行那次发布、上一版带文件：按 [`row_publish_plan_on_file_version`] 发。
pub(super) fn publish_rows_on_file_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance_table: InstanceTableRewrite,
    identity: RowPublishIdentity,
) -> Result<TransactionOutput, PublishError> {
    publish_version(
        pool,
        allocator,
        row_publish_plan_on_file_version(previous, instance_table, &identity),
        Some(previous),
    )
}

/// 暖机的一次空发布接在带文件的现行那一版后面时的计划：重写固定点单元（记账树存在，D16（发布语义） 已定项 9），不写文件内容、不碰 inode 树、
/// 实例表照抄。取号之后的发布与取号之前的预演读的是同一张计划；会话的推环（`mounted_session`）发的也是这一张（F 不变）。
pub(crate) fn empty_publish_plan_after_file_version<'plan>(
    current_file_version: &TransactionOutput,
    instance: InstanceGeneration,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: checkpoint_txg_of_the_planned_publish_after(current_file_version.root.checkpoint_txg),
        counter: current_file_version.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current_file_version
            .highest_transaction_number_in_this_instance,
        instance,
        back_chain: back_chain_of(&current_file_version.record_bytes),
        file: None,
        // 暖机那几次空发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current_file_version.root.instance_table),
        tree_birth_txg: current_file_version.tree_birth_txg(),
        // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
        tree_identifier_watermark: current_file_version.root.tree_identifier_watermark,
        rollback_floor: current_file_version.root.rollback_floor,
    }
}

/// 暖机的一次空发布接在树表 0 条的现行那一版后面时的计划（零单元，D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）。
pub(super) fn empty_publish_plan_after_version_without_file(
    current_version_without_file: &VersionWithoutFilePublishOutput,
    instance: InstanceGeneration,
) -> ZeroUnitPublishPlan {
    ZeroUnitPublishPlan {
        txg: checkpoint_txg_of_the_planned_publish_after(
            current_version_without_file.root.checkpoint_txg,
        ),
        counter: current_version_without_file.record.counter + 1,
        instance,
        back_chain: back_chain_of(&current_version_without_file.record_bytes),
        rollback_floor: current_version_without_file.root.rollback_floor,
        // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
        tree_identifier_watermark: current_version_without_file.root.tree_identifier_watermark,
    }
}

/// 暖机的一次空发布，接在现行那一版后面：带文件的一版重写固定点单元（记账树存在，D16（发布语义） 已定项 9），
/// 树表 0 条的一版写零个单元（同一条已定项「树表 0 条 ⇒ 零单元」）。
pub(super) fn publish_empty_after<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    current: &PoolVersion,
    instance: InstanceGeneration,
) -> Result<PoolVersion, PublishError> {
    match current {
        PoolVersion::WithFile(current_file_version)
        | PoolVersion::WithTreesAndNoFile(current_file_version) => publish_version(
            pool,
            allocator,
            empty_publish_plan_after_file_version(current_file_version, instance),
            Some(Arc::as_ref(current_file_version)),
        )
        .map(PoolVersion::of_a_version_of_trees),
        PoolVersion::WithoutFile(current_version_without_file) => {
            let published = publish_without_units(
                pool,
                &current_version_without_file.root,
                empty_publish_plan_after_version_without_file(
                    current_version_without_file,
                    instance,
                ),
            )?;
            // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`），
            // 记之前照 E2 重读挂载时读不出的那一槽。
            record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
                &*pool.devices,
                pool.parameters,
                allocator,
                published.root.checkpoint_txg,
            );
            Ok(PoolVersion::WithoutFile(published))
        }
    }
}

/// 带文件的一版的一次发布真取到的落点，按取落点的次序：这次重写的每个角色（`TransactionOutput::rewritten`）。
pub(super) fn placements_taken_by_a_file_version(
    output: &TransactionOutput,
) -> PlacementsTakenByOnePublish {
    output
        .rewritten
        .iter()
        .map(|role| (*role, output.unit(*role).slot))
        .collect()
}

/// 一次发布真取到的落点，按取落点的次序：带文件的一版见 [`placements_taken_by_a_file_version`]；
/// 树表 0 条的一版是这次那条记录的点名项：写行那次按取落点的次序点名实例表链各片（尾片先）与分配记录树节点
/// （`transaction::publish_instance_table_on_version_without_file`；第 1 片起的指针不在根记录里，只在记录的点名项与上一片的链指针里），
/// 零单元发布一项都不点名、一个都没有。
pub(super) fn placements_taken_by(version: &PoolVersion) -> PlacementsTakenByOnePublish {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            placements_taken_by_a_file_version(output)
        }
        PoolVersion::WithoutFile(version_without_file) => {
            // 点名项按取落点的次序排在这次发布的各条记录里（末条再跨记录时前几条在 `earlier_records_of_this_publish`，
            // D23（journal 的角色与格式） 已定项 17），按记录的次序接起来就是整次发布的点名项。
            let named: Vec<&crate::journal::NamedUnit> = version_without_file
                .earlier_records_of_this_publish
                .iter()
                .map(|written| &written.record)
                .chain(std::iter::once(&version_without_file.record))
                .flat_map(|record| record.named.iter())
                .collect();
            assert_eq!(
                named.len(),
                version_without_file.rewritten.len(),
                "树表 0 条的一版上一次发布的点名项与它写出的角色一一对应（写行那次按同一张角色清单点名，零单元发布两边都空）"
            );
            version_without_file
                .rewritten
                .iter()
                .copied()
                .zip(named)
                .map(|(role, named_unit)| {
                (
                    role,
                    slot_shared_by_both_location_entries(&named_unit.locations).expect(
                        "这个进程这次发布刚写的指针，两条位置条目按一个落点写（两盘同槽，`PoolWriter::location_entries`）",
                    ),
                )
            })
            .collect()
        }
    }
}

/// 暖机要推的那几次空发布，按它们的 checkpoint_txg 列出来（D16（发布语义） 已定项 8 戊）：从写行那次发布的 txg 起逐个加一，
/// 按根环落点公式看落哪块盘，直到本实例的根覆盖每块盘，至多根环的区域数那么多次。
/// 纯算——只读区域归属表与池里的盘，不碰盘、不碰分配器、不看分配记录，所以取号之前算得出来。
/// 取号之前算准入与取号之后推发布共用这一份计划（`establish_instance` 按它推），两处的次数因此必定相同：
/// 各算各的时只要有一处多算一次，准入就罩不住实际推的那几次。
///
/// # Errors
/// 下一个 txg 越过 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条；取号之前，一个写都没发）。
pub(super) fn warm_up_publish_txgs(
    parameters: &MakeFilesystemParameters,
    all_devices: &[DeviceIdentity],
    row_publish_txg: CheckpointTxg,
) -> Result<Vec<CheckpointTxg>, MountError> {
    let device_of_txg = |txg: CheckpointTxg| {
        parameters.region_devices[usize::try_from(
            target_for_publish(txg, parameters.geometry.root_ring_slots_per_region).region,
        )
        .expect("区域号")]
    };
    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish_txg)];
    let mut warm_up_publishes: Vec<CheckpointTxg> = Vec::new();
    let mut latest_txg = row_publish_txg;
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next_txg = checkpoint_txg_after_the_version(latest_txg)?;
        let device = device_of_txg(next_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        warm_up_publishes.push(next_txg);
        latest_txg = next_txg;
    }
    Ok(warm_up_publishes)
}

/// 这次挂载要写的行：给 [max(上一个实例, 1), 要取的号) 里每个实例各一行——上一个实例 (i, T, W)，
/// 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。纯算，取号之前就列得出来。
pub(super) fn instance_rows_to_write(
    previous_row: &PreviousInstanceRow,
    instance_to_acquire: InstanceGeneration,
) -> Vec<InstanceRow> {
    let first_row_instance = previous_row.instance.0.max(1);
    (first_row_instance..instance_to_acquire.0)
        .map(|row_instance| {
            if row_instance == previous_row.instance.0 {
                InstanceRow {
                    instance: InstanceGeneration(row_instance),
                    selected_root_txg: previous_row.selected_root_txg,
                    applied_transaction_high_water: previous_row.applied_transaction_high_water,
                }
            } else {
                InstanceRow {
                    instance: InstanceGeneration(row_instance),
                    selected_root_txg: CheckpointTxg(0),
                    applied_transaction_high_water: 0,
                }
            }
        })
        .collect()
}

/// 一串发布里有一次失败了：错，连同这一串的写入口交得出的账——`writes_before_the_first_publish` 是第一次发布之前、不属于任何一次发布的写
/// （抬 F 那一串先写系统配置那一步；可写挂载那一串给空账），`writes_of_persisted_publishes` 是这一串里已经落盘的那几次各自的写
/// （按先后），失败那一次落盘阶段已记的写从写入口的失败账取。可写挂载与抬 F 共用这一处。
pub(super) fn publish_sequence_failed<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    writes_before_the_first_publish: WritesByStructureKind,
    writes_of_persisted_publishes: Vec<WritesByStructureKind>,
    cause: PublishError,
) -> Box<PublishSequenceFailed> {
    Box::new(PublishSequenceFailed {
        cause,
        writes_before_the_first_publish,
        writes_of_persisted_publishes,
        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
    })
}

/// 取号之后的一次发布失败了：错，连同这次挂载的写入口交得出的账——`persisted` 是这次挂载里已经落盘的那几次（按先后）。
pub(super) fn publish_failed_after_acquisition<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    persisted: &[&PoolVersion],
    cause: PublishError,
) -> MountError {
    MountError::Publish(publish_sequence_failed(
        pool,
        WritesByStructureKind::NOTHING_WRITTEN,
        persisted
            .iter()
            .map(|version| match version {
                PoolVersion::WithoutFile(version_without_file) => {
                    version_without_file.writes.clone()
                }
                PoolVersion::WithFile(file_version)
                | PoolVersion::WithTreesAndNoFile(file_version) => file_version.writes.clone(),
            })
            .collect(),
        cause,
    ))
}

/// 所选那一版在 journal 里的位置：它那次发布的末条记录的 jsn（调用方从环里认出来的那一条，认不出时它顶上的那一条）；
/// 一条都拿不出来（只做过 mkfs 的池，环里没有记录）时取 (这一版的实例代号, 0)。每块盘在一次发布之后轮换的系统配置写的是
/// 同一对 (实例代号, 末条计数器)（`transaction::CommitStep::RotateSystemConfigurationSlots`），逐盘判「落后」拿它比。
pub(super) fn journal_position_of_the_selected_version(
    root: &RootRecord,
    record_standing_for_root: Option<&crate::journal::JournalRecord>,
) -> JournalSequenceNumber {
    record_standing_for_root.map_or(
        JournalSequenceNumber {
            instance_generation: root.instance,
            counter: 0,
        },
        |record| JournalSequenceNumber {
            instance_generation: record.instance,
            counter: record.counter,
        },
    )
}

#[cfg(test)]
mod versions_and_unit_bytes_shared_instead_of_copied_tests {
    //! 判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节 Q2.1 甲档与 Q2.2 `Arc<TransactionOutput>` 那一小块：
    //! 照抄的单元与上一版共用同一份字节，版本列表、挂载交回值与现行那一版共用同一份 `TransactionOutput`。判的是「指着同一份」
    //! （`Arc::ptr_eq`），内容相同的拷贝判红；每条另带一个对照，证明这一判分得出共用与不共用。
    use std::sync::Arc;

    use super::empty_publish_plan_after_file_version;
    use super::pool_in_memory_after_the_first_file::pool_after_the_first_file;
    use super::{mount_writable, ParametersAndDeviceTableOfTheMount, ShadowLedger};
    use crate::address::{DataUnitIndexInFile, InodeNumber};
    use crate::compaction_plan::CompactionOfThisMount;
    use crate::mounted_session::{MountedSession, UserChange};
    use crate::transaction::{
        publish_version, FirstFile, PoolVersion, PoolWriter, PublishedUnit, TransactionUnit,
        FIRST_INODE_NUMBER,
    };

    /// 这几条用例里改动的写入时刻：晚于池里第一个文件的写入时刻。
    const WRITE_TIME_SECONDS_OF_THE_CHANGE: u64 = 1_788_000_100;

    /// 池里第一个文件之后、这个进程里接着它的会话（没经过可写挂载：盘表读盘造，整理状态从新挂载起）。
    fn session_after_the_first_file() -> (
        MountedSession,
        Vec<(
            crate::address::DeviceIdentity,
            super::shadow_ledger_exempts_and_isolates_slot_by_slot_tests::SparseDeviceInMemory,
        )>,
    ) {
        let pool = pool_after_the_first_file();
        let parameters_and_device_table =
            ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&pool.devices)
                .expect("mkfs 与第一个文件写过的两块盘上择得出系统配置、盘表与它一致");
        let session = MountedSession {
            allocator: pool.allocator,
            current: PoolVersion::of_a_version_of_trees(pool.current),
            instance: pool.instance,
            shadow_ledger: ShadowLedger::On,
            parameters_and_device_table,
            compaction: CompactionOfThisMount::of_a_new_mount(),
        };
        (session, pool.devices)
    }

    /// 甲档：空发布照抄的每一个单元（第一个文件的数据单元、extent 树根、inode 树、实例表链）与上一版那一份指着同一份字节；
    /// 对照：这次重写的树表是新装的字节，不与上一版共用。
    #[test]
    fn an_empty_publish_shares_the_bytes_of_every_unit_it_carries_with_the_version_before() {
        let mut pool = pool_after_the_first_file();
        let previous = pool.current.clone();
        let next = {
            let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
            publish_version(
                &mut writer,
                &mut pool.allocator,
                empty_publish_plan_after_file_version(&previous, pool.instance),
                Some(&previous),
            )
            .expect("4 GiB 两盘上空发布落得下")
        };
        let carried: Vec<&PublishedUnit> = next
            .units
            .iter()
            .filter(|unit| !next.rewritten.contains(&unit.identity))
            .collect();
        assert!(
            carried
                .iter()
                .any(|unit| unit.identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST)),
            "前提：空发布照抄第一个文件的数据单元（照抄的那几个里有它）"
        );
        for unit in &carried {
            assert!(
                Arc::ptr_eq(&unit.bytes, &previous.unit(unit.identity).bytes),
                "照抄的 {:?} 与上一版那一份指着同一份字节，不是内容相同的拷贝",
                unit.identity
            );
        }
        assert!(
            next.rewritten.contains(&TransactionUnit::TreeTable),
            "前提：空发布重写树表"
        );
        assert!(
            !Arc::ptr_eq(
                &next.unit(TransactionUnit::TreeTable).bytes,
                &previous.unit(TransactionUnit::TreeTable).bytes
            ),
            "对照：重写的树表是这次装的字节，与上一版不共用（共用这一判分得出两种）"
        );
    }

    /// `Arc<TransactionOutput>`：一次覆盖写经会话发成，交回的版本列表最后那一版与会话的现行那一版指着同一份；
    /// 对照：上一次改动之前的现行那一版与这次的不是同一份。
    #[test]
    fn user_change_hands_back_the_version_it_published_as_the_one_the_session_holds_as_current() {
        let (mut session, mut devices) = session_after_the_first_file();
        let current_before = Arc::clone(
            session
                .current
                .shared_version_of_trees()
                .expect("第一个文件之后现行那一版有树"),
        );
        let content = [0x5A_u8; 2000];
        let published = session
            .publish_user_change(
                &mut devices,
                UserChange::Overwrite(FirstFile {
                    content: &content,
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE,
                }),
            )
            .unwrap_or_else(|refusal| panic!("4 GiB 两盘上覆盖写经会话发得成：{refusal:?}"));
        assert_eq!(
            published.versions_published.len(),
            1,
            "前提：队列空、没推抬 F，这次改动只发成一版"
        );
        let current_after = session
            .current
            .shared_version_of_trees()
            .expect("覆盖写之后现行那一版有树");
        assert!(
            Arc::ptr_eq(&published.versions_published[0], current_after),
            "交回的那一版与会话的现行那一版指着同一份，不是内容相同的拷贝"
        );
        assert!(
            !Arc::ptr_eq(&current_before, current_after),
            "对照：改动之前的现行那一版与之后的不是同一份"
        );
    }

    /// `Arc<TransactionOutput>`：经会话 unlink 唯一那个文件（意图那一次，再走空到最后一批），走空交回的最后一步与会话的现行那一版
    /// 指着同一份（走空每发成一步不再拷一整版）；对照：意图那一版与现行那一版不是同一份。
    #[test]
    fn an_unlink_through_the_session_hands_back_the_last_step_of_the_walk_as_the_session_current() {
        let (mut session, mut devices) = session_after_the_first_file();
        let published = session
            .publish_user_change(
                &mut devices,
                UserChange::Unlink {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE,
                },
            )
            .unwrap_or_else(|refusal| panic!("unlink 唯一那个文件经会话发得成：{refusal:?}"));
        assert!(
            published.versions_published.len() >= 2,
            "前提：意图那一次之后至少走空一步，实际 {} 版",
            published.versions_published.len()
        );
        let current_after = session
            .current
            .shared_version_of_trees()
            .expect("unlink 之后现行那一版有树（文件集合为空）");
        assert!(
            Arc::ptr_eq(
                published
                    .versions_published
                    .last()
                    .expect("上面判过至少两版"),
                current_after
            ),
            "走空交回的最后一步与会话的现行那一版指着同一份，不是内容相同的拷贝"
        );
        assert!(
            !Arc::ptr_eq(&published.versions_published[0], current_after),
            "对照：意图那一版与走空之后的现行那一版不是同一份"
        );
    }

    /// `Arc<TransactionOutput>`：同一组盘可写重挂，挂载交回值里最后那一次暖机与挂载交回的现行那一版指着同一份；
    /// 对照：写行那一版与现行那一版不是同一份（暖机至少一次）。
    #[test]
    fn writable_remount_hands_back_its_last_warm_up_publish_as_the_current_version() {
        let mut pool = pool_after_the_first_file();
        let mounted = mount_writable(&pool.parameters, &mut pool.devices)
            .expect("第一个文件之后可写重挂做得成");
        assert!(
            mounted.output.deletion_queue_continuation.is_empty()
                && mounted.output.space_admission.floor_raises().is_empty(),
            "前提：队列空、准入够，挂载不续做、不推抬 F"
        );
        let last_warm_up_publish = mounted
            .output
            .warm_up_publishes
            .last()
            .expect("前提：本实例的根要覆盖两块盘，写行之后至少暖机一次");
        assert_eq!(
            last_warm_up_publish.root(),
            mounted.current.root(),
            "前提：暖机之后这次挂载没再发布（没续做、没整理、清扫没发），现行那一版就是最后一次暖机"
        );
        let current = mounted
            .current
            .shared_version_of_trees()
            .expect("第一个文件之后的一版有树");
        assert!(
            Arc::ptr_eq(
                last_warm_up_publish
                    .shared_version_of_trees()
                    .expect("有树的一版上暖机交回有树的一版"),
                current
            ),
            "挂载交回的最后一次暖机与现行那一版指着同一份，不是内容相同的拷贝"
        );
        assert!(
            !Arc::ptr_eq(
                mounted
                    .output
                    .row_publish
                    .shared_version_of_trees()
                    .expect("有树的一版上写行交回有树的一版"),
                current
            ),
            "对照：写行那一版与现行那一版不是同一份"
        );
    }
}
