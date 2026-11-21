//! 恢复之后从盘上重建上一版（[`PreviousVersion`]）与建立新实例要带的东西（[`InstanceStart`]）：写行那次发布的实例表链怎么重写、
//! 新实例第一次发布的 txg 与树 ID 水位。

use super::*;

/// 读一条根指着的实例表那一读没读通，按失败种类报成两个成员之一（C866 改法甲：字节丢了与内容拒收各自的成员，
/// `recovery::RecoveryFailure::failure_kind_for_the_fall_back`）。
pub(super) fn instance_table_refusal(failure: &RecoveryFailure) -> MountError {
    match failure.failure_kind_for_the_fall_back() {
        crate::recovery::FailureKindForTheFallBack::BytesLost => {
            MountError::InstanceTableUnreadable
        }
        crate::recovery::FailureKindForTheFallBack::ContentRefused => {
            MountError::InstanceTableMalformed
        }
    }
}

fn map_rebuild_failure(failure: RebuildVersionFailure) -> MountError {
    match failure {
        RebuildVersionFailure::Walk(recovery_failure) => MountError::Recovery(recovery_failure),
        RebuildVersionFailure::NoRecordStandingForFileVersion => {
            MountError::FileVersionWithoutAnyJournalRecord
        }
    }
}

/// 恢复之后从盘上重建出来的上一版，带文件的连同写行要接在后面的那一版实例表。
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只有一个，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
#[allow(
    clippy::enum_variant_names,
    reason = "三个成员与 PoolVersion、RebuiltVersion 同名（D8 已定项 16 第 5 条：第三个成员三处同形），都以 File 结尾是条款的叫法"
)]
/// 每个成员另带建它时按位置条目读单元读到只坏一份的那几个单元的位置条目（`units_read_with_one_copy_failing`，一个单元一组；
/// [`PoolReaderCountingUnitReadsWithOneCopyFailing::locations_of_the_units_read_with_one_copy_failing`]）：可写挂载交回之前按挂载之后那一版
/// 认成映射 key，交给会话下一次用户改动之后另发的那一批（用户 2026-11-05 定「可写挂载时搬」）。
pub(super) enum PreviousVersion {
    /// 树表 0 条：这一版只有根自己指着的实例表与树表两个单元。写行要在这一版那张实例表后面接行，所以连表一起解出来。
    WithoutFile {
        root: RootRecord,
        table: InstanceTableChain,
        units_read_with_one_copy_failing: Vec<[LocationEntry; 2]>,
    },
    WithFile {
        output: TransactionOutput,
        table: InstanceTableChain,
        units_read_with_one_copy_failing: Vec<[LocationEntry; 2]>,
    },
    /// 有树、文件集合为空（D8（核心索引结构） 已定项 16 第 5 条，`RebuiltVersion::WithTreesAndNoFile`）：写行与暖机照带文件的一版那样做，
    /// 发出来的一版按 `PoolVersion::of_a_version_of_trees` 归成员。
    WithTreesAndNoFile {
        output: TransactionOutput,
        table: InstanceTableChain,
        units_read_with_one_copy_failing: Vec<[LocationEntry; 2]>,
    },
}

impl PreviousVersion {
    /// 建这一版时读到只坏一份的那几个单元的位置条目（[`Self::record_the_units_read_with_one_copy_failing_while_building_on_it`] 记的）。
    pub(super) fn units_read_with_one_copy_failing_while_building_on_it(
        &self,
    ) -> &[[LocationEntry; 2]] {
        match self {
            PreviousVersion::WithoutFile {
                units_read_with_one_copy_failing,
                ..
            }
            | PreviousVersion::WithFile {
                units_read_with_one_copy_failing,
                ..
            }
            | PreviousVersion::WithTreesAndNoFile {
                units_read_with_one_copy_failing,
                ..
            } => units_read_with_one_copy_failing,
        }
    }

    /// 记下建这一版时读到只坏一份的那几个单元的位置条目（建完、落定建的是哪一版之后记一次；退到的那一版建不出、又建所选那一版时，
    /// 两趟读到的都在里面，认 key 时按挂载之后那一版滤）。
    pub(super) fn record_the_units_read_with_one_copy_failing_while_building_on_it(
        &mut self,
        locations: Vec<[LocationEntry; 2]>,
    ) {
        match self {
            PreviousVersion::WithoutFile {
                units_read_with_one_copy_failing,
                ..
            }
            | PreviousVersion::WithFile {
                units_read_with_one_copy_failing,
                ..
            }
            | PreviousVersion::WithTreesAndNoFile {
                units_read_with_one_copy_failing,
                ..
            } => *units_read_with_one_copy_failing = locations,
        }
    }

    /// 这一版的实例表链：全部行（写行时要在它后面接上这次的行，取号之前的准入也按它的行数算）与每一片的指针
    /// （写行那次发布整条链重写，这几片逐片释放）。两臂都有表：树表 0 条那一版的表是根记录直接指着的那条链
    /// （mkfs 种下的，或上一次挂载写行重写的）。
    pub(super) fn instance_table_chain(&self) -> &InstanceTableChain {
        match self {
            PreviousVersion::WithoutFile { table, .. }
            | PreviousVersion::WithFile { table, .. }
            | PreviousVersion::WithTreesAndNoFile { table, .. } => table,
        }
    }

    /// 有树的那一版（带文件的、有树而文件集合为空的）；树表 0 条的是 `None`（空间准入的 ckpt_cost 按它算，
    /// `admission::checkpoint_cost_of_the_version_to_build_on`）。
    pub(super) fn version_of_trees(&self) -> Option<&TransactionOutput> {
        match self {
            PreviousVersion::WithoutFile { .. } => None,
            PreviousVersion::WithFile { output, .. }
            | PreviousVersion::WithTreesAndNoFile { output, .. } => Some(output),
        }
    }

    /// 这一版的单元（角色、落点、验得过的那一份字节）：可写挂载在取号之前逐盘核每块盘带不带它们
    /// （`devices_without_the_selected_version`）。落点取第一条位置条目的槽——第一版两盘同槽（D2（RAID 条带策略） 已定项 10），
    /// 发布路径照抄一个单元时也按这一个落点对待它。
    ///
    /// 带文件的一版就是重建出来的 `units`（全部角色；提示与映射都读不出的数据单元字节是空的，逐盘核时跳过它）。
    /// 树表 0 条的一版是根记录直接指着的几个单元：实例表第 0 片、树表、写过行的一版那棵分配记录树的每个节点
    /// （与 `placements_referenced_by_root` 树表 0 条那一臂同一份清单，字节按位置条目读验得过的那一份）。
    /// 两臂都不列实例表链第 1 片起的各片：每次写行整条链 COW 重写（D18（块里携带什么信息） 已定项 11），后面各片与第 0 片同一次发布写出，
    /// 停在那次发布之前的盘缺它们，就也缺第 0 片。
    ///
    /// # Errors
    /// 树表 0 条那一版的实例表、树表或分配记录树这一次读不出、解不开（重建时读得出，两次读之间的瞬时读错）⇒ `Recovery`：
    /// 核不了就不放行，在任何写之前返回。
    pub(super) fn units<Reader: PoolReader>(
        &self,
        reader: &Reader,
        unit_area_start: UnitAreaStart,
    ) -> Result<Cow<'_, [PublishedUnit]>, MountError> {
        match self {
            PreviousVersion::WithFile { output, .. }
            | PreviousVersion::WithTreesAndNoFile { output, .. } => {
                Ok(Cow::Borrowed(&output.units))
            }
            PreviousVersion::WithoutFile { root, .. } => {
                let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
                let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
                let mut units = vec![
                    PublishedUnit {
                        slot: root.instance_table.locations[0].slot,
                        identity: TransactionUnit::InstanceTable,
                        bytes: Arc::from(read_unit_via_locations(
                            reader,
                            &root.instance_table.locations,
                            data_unit_bytes,
                        )?),
                    },
                    PublishedUnit {
                        slot: root.tree_table.locations[0].slot,
                        identity: TransactionUnit::TreeTable,
                        bytes: Arc::from(read_unit_via_locations(
                            reader,
                            &root.tree_table.locations,
                            node_bytes,
                        )?),
                    },
                ];
                if let Some(tree) =
                    allocation_records_of_version_without_file_in_the_unit_area_starting_at(
                        reader,
                        unit_area_start,
                        root,
                    )?
                {
                    for (node, pointer) in &tree.version.nodes {
                        units.push(PublishedUnit {
                            slot: pointer.locations[0].slot,
                            identity: role_of_allocation_record_tree_node(*node),
                            bytes: Arc::from(read_unit_via_locations(
                                reader,
                                &pointer.locations,
                                node_bytes,
                            )?),
                        });
                    }
                }
                Ok(Cow::Owned(units))
            }
        }
    }
}

/// 写行那次发布的实例表链怎么重写：这一版那张表的行后面接上这次写的行，被换下的是这一版的整条链
/// （D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）。取号之前的准入与写行那次发布读的都是它，两处同一份输入。
pub(super) fn instance_table_rewrite_of_the_row_publish(
    table: &InstanceTableChain,
    rows_written: &[InstanceRow],
) -> InstanceTableRewrite {
    let mut rows = table.records.rows.clone();
    rows.extend_from_slice(rows_written);
    InstanceTableRewrite {
        rows,
        replaced_chain: table.page_pointers.clone(),
    }
}

/// 重建上一版：树表 0 条就留根加它指着的那张实例表；带文件的连同重建出来的单元。两臂的实例表都从根记录里那条指针沿链读
/// （D18（块里携带什么信息） 已定项 11）——只解 `TransactionUnit::InstanceTable` 那一个单元的字节只有第 0 片，
/// 多于一片的表会被按前半张判准入、接行。读盘经 `reader`：可写挂载交的是套着数「只坏一份」的那一层
/// （[`PoolReaderCountingUnitReadsWithOneCopyFailing`]，D4（校验和位置） 已定项 4「读到只坏一份时计数」）。
pub(super) fn rebuild_previous_version(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    record_standing_for_root: Option<crate::journal::JournalRecord>,
) -> Result<PreviousVersion, MountError> {
    let rebuilt = rebuild_version_in_the_unit_area_starting_at(
        reader,
        unit_area_start,
        root,
        record_standing_for_root,
    )
    .map_err(map_rebuild_failure)?;
    let table = instance_table_chain_of_root(reader, root)
        .map_err(|failure| instance_table_refusal(&failure))?;
    match rebuilt {
        RebuiltVersion::WithoutFile => Ok(PreviousVersion::WithoutFile {
            root: *root,
            table,
            units_read_with_one_copy_failing: Vec::new(),
        }),
        RebuiltVersion::WithFile(output) => Ok(PreviousVersion::WithFile {
            output,
            table,
            units_read_with_one_copy_failing: Vec::new(),
        }),
        RebuiltVersion::WithTreesAndNoFile(output) => Ok(PreviousVersion::WithTreesAndNoFile {
            output,
            table,
            units_read_with_one_copy_failing: Vec::new(),
        }),
    }
}

/// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。这一遍直接读根环，`chosen_root`（读阶段择到的那条根，判退之前的那一条：择根取环里 txg 最大的，
/// 读阶段读到的根都不超过它）再取一次 max：两次读环之间的瞬时读错（择根那一遍读到了它、这一遍没读到）不把下界拉到它之下，判退退掉的根同样算。
/// 所选根施加前缀之后的那一版不另收：它的 txg 来自 `records` 里的记录，已在记录那一半里。
///
/// # Errors
/// 那个最大值已是 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条：盘上读来的 txg，不 panic）。
pub(super) fn first_txg_of_new_instance<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    chosen_root: &RootRecord,
) -> Result<CheckpointTxg, MountError> {
    let highest_record_txg = records
        .values()
        .map(|record| record.checkpoint_txg)
        .max()
        .unwrap_or(CheckpointTxg(0));
    let highest_ring_txg = highest_root_txg(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .unwrap_or(CheckpointTxg(0))
    .max(chosen_root.checkpoint_txg);
    checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))
}

/// 接在 txg 为 `version` 的那一版后面的下一个 txg（它加一），挂载一侧（可写挂载、抬 F、正常卸载）用。
///
/// # Errors
/// `version` 已是 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条）。
pub(super) fn checkpoint_txg_after_the_version(
    version: CheckpointTxg,
) -> Result<CheckpointTxg, MountError> {
    version.0.checked_add(1).map(CheckpointTxg).ok_or(
        MountError::SequenceNumberPastTheTopOfItsRange(
            SequenceNumberAtTheTopOfItsRange::CheckpointTxg(version),
        ),
    )
}

/// 本实例第一次发布要带的树 ID 水位（D8（核心索引结构） 已定项 8 ②「根环里全部根记录该字段的 max」）：
/// 根环里全部自证过的根与 `records`（环里全部自证通过的记录）各自带的水位取 max，再与这次要接在后面的那一版、读阶段择到的那条根
/// （判退之前的那一条）各自带的取 max——这两条都是这次挂载从环里读到过的；再取一次是为了两次读环之间的瞬时读错
/// （之前那一遍读到了它、这一遍没读到）不把水位拉低，判退退掉的根发过的号同样不许重发（`first_txg_of_new_instance` 同形）。
/// 读不出的根怎么算见 `highest_tree_identifier_watermark_in_the_ring`。
pub(super) fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    version_to_build_on: &RootRecord,
    root_chosen_by_the_read_stage: &RootRecord,
) -> u64 {
    highest_tree_identifier_watermark_in_the_ring(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        records,
    )
    .map_or(version_to_build_on.tree_identifier_watermark, |ring| {
        ring.max(version_to_build_on.tree_identifier_watermark)
    })
    .max(root_chosen_by_the_read_stage.tree_identifier_watermark)
}
