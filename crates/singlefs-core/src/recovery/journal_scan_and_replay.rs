//! 全环扫描 journal 与重放（[`scan_journal`]、[`replay_journal`]）：反向链对盘上前一条的判定，
//! 点名单元逐项验，由记录的新根段拼出根。

use super::*;

/// 全环扫描：每盘每个记录槽都读（有提示时只读提示的那些），两份镜像任一份自证过即算在；fsid 不符的不算数。
/// 没有提示时逐槽列偏移只列到那块盘的末尾：环长是盘上读来的 8 字节，读者只按「环末端不越过同一槽记着的单元区起点、在飞上限装得进 4 字节」
/// 判它（`system_configuration_values_this_reader_accepts`），不按盘的字节数判；盘末尾之后的槽本来就读不出（`PoolReader::read` 交空），
/// 不列它们，扫环的内存就不由那 8 字节定。
#[must_use]
pub fn scan_journal(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> BTreeMap<(InstanceGeneration, u64), JournalRecord> {
    let expected_filesystem_identifier =
        unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier);
    let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let mut scanned_records_in_read_order: Vec<JournalRecord> = Vec::new();
    for device in reader.device_identities() {
        let ring_bytes_inside_the_device = reader
            .device_size_in_bytes(device)
            .map_or(0, |device_bytes| {
                ring_bytes.min(device_bytes.saturating_sub(ring_start.0))
            });
        let offsets = reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
            .unwrap_or_else(|| {
                (0..ring_bytes_inside_the_device / JOURNAL_RECORD_BYTES)
                    .map(|record_index| {
                        DeviceOffsetInBytes(ring_start.0 + record_index * JOURNAL_RECORD_BYTES)
                    })
                    .collect()
            });
        for offset in offsets {
            let Some(bytes) = reader.read(device, offset, record_bytes) else {
                continue;
            };
            if let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) {
                scanned_records_in_read_order.push(record);
            }
        }
    }
    // 同一 (实例代号, 计数器) 先到先得（按上面的盘次序、环内偏移次序，第一份读到的留下）：留哪一份在
    // `singlefs_replay_prefix::first_copy_of_each_record_key` 里（Verus 验的是那一份），这里照它交回的位置取。
    let scanned_record_keys: Vec<(u32, u64)> = scanned_records_in_read_order
        .iter()
        .map(|record| (record.instance.0, record.counter))
        .collect();
    let first_copies = first_copy_of_each_record_key(&scanned_record_keys);
    let mut records = BTreeMap::new();
    for (position, record) in scanned_records_in_read_order.into_iter().enumerate() {
        if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
            records.insert((record.instance, record.counter), record);
        }
    }
    records
}

/// 一条记录的反向链对不对得上本实例内逻辑前一条（I-8.6（反向链算法））。封闭集合，`match` 不写通配臂。
enum BackChainJudgement {
    /// 前一条在盘上，它的头算出来的链值等于这一条的反向链。
    Holds,
    /// 前一条在盘上，链值不等：这一条不进重放前缀（I-8.6「不等的记录不进重放前缀」）。
    Broken,
    /// 本实例内逻辑前一条不在盘上（扫环时没读到，或重读那一槽时每一份都解不出它）：判不了，不拿它断前缀。
    PreviousRecordNotOnDisk,
}

/// 判一条记录的反向链（I-8.6（反向链算法）；D23（journal 的角色与格式） 已定项 8 / 已定项 10 / 已定项 19 ②）：
/// 本实例内逻辑前一条 = 同一实例、计数器小 1 的那一条；链值 = CRC-32C(它在盘上那一份的 496 字节头，`header_csum` 按 0 参与)。
/// 按盘上的原样字节算、不按解出来的字段重写（重写会把头里读者不看的字节抹成 0），与池级 checker 判 I-8.6 同一个输入；
/// 取的是 [`PoolReader::device_identities`] 次序里第一份解得出、且与扫环时收下的那一条相同的镜像——扫环按同一次序收记录。
/// 前一条不在盘上不判：链是按实例的，跨实例边界本来不比，前缀里的记录都接在所选根覆盖的那一条之后，
/// 读不出那一条时链首另有「本次发布内序号为 1」一关（D23（journal 的角色与格式） 已定项 14 注 1）。
fn judge_back_chain_against_the_previous_record_on_disk(
    reader: &dyn PoolReader,
    record: &JournalRecord,
    ring_bytes: u64,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> BackChainJudgement {
    let Some(previous_counter) = record
        .counter
        .checked_sub(1)
        .filter(|previous_counter| *previous_counter >= FIRST_JOURNAL_COUNTER)
    else {
        return BackChainJudgement::PreviousRecordNotOnDisk;
    };
    let Some(previous) = records.get(&(record.instance, previous_counter)) else {
        return BackChainJudgement::PreviousRecordNotOnDisk;
    };
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let offset = record_offset(previous_counter, ring_bytes);
    // 迭代次数的上界是池里的盘数；第一份解得出、与扫到的那一条相同的镜像就是出口。
    for device in reader.device_identities() {
        let Some(bytes) = reader.read(device, offset, record_bytes) else {
            continue;
        };
        if JournalRecord::parse(&bytes, previous.filesystem_identifier).as_ref() == Some(previous) {
            return if back_chain_of(&bytes) == record.back_chain {
                BackChainJudgement::Holds
            } else {
                BackChainJudgement::Broken
            };
        }
    }
    BackChainJudgement::PreviousRecordNotOnDisk
}

/// [`replay_journal`] 交给取前缀核的「新根从哪来」：核不看它，施加了发布就交回最后施加的那次发布末条带的那一项，
/// 一次都没施加就交回调用方给的那一项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NewRootSource {
    ChosenRootItself,
    NewRootSegmentOfTheRecordAtKeyOrderIndex(usize),
}

/// 所选根的 checkpoint_txg 是 u64 的最大值时 [`replay_journal`] 报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
pub const CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR: &str =
    "所选根的 checkpoint_txg 是 u64 的最大值：接在它后面的那次发布取不到 txg";

/// 一条记录点名的每个单元的两份（两条位置条目）都读得出、整单元 CRC-32C 都等于条目里的（前缀第四条「两份都验过才施加」，
/// D23（journal 的角色与格式） 已定项 14）；点名项的单元种类不认识就算验不过。只给施加前验点名单元（[`replay_journal`]）用；
/// 可写挂载判「水位之上一次缺记录的发布点名的单元验不过」用的是另一份（[`every_named_unit_of_the_record_has_a_copy_that_verifies`]）。
pub(crate) fn every_named_unit_of_the_record_verifies<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    record: &JournalRecord,
) -> bool {
    record.named.iter().all(|named| {
        let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
            return false;
        };
        named
            .locations
            .iter()
            .all(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))
    })
}

/// 一条记录点名的每个单元都至少有一份（一条位置条目）读得出、整单元 CRC-32C 等于条目里的；点名项的单元种类不认识就算验不过。
/// 可写挂载判「水位之上一次缺记录的发布点名的单元验不过」（F1 乙，`mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`）
/// 只用这一份：某个点名单元两份都验不过才算验不过（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行与第三节第 1 条，
/// 攻方 `F1_ANY`）。一份副本坏是单故障（D22（单元原子性怎么合成） 已定项 8 射程），按「两份都验过」判会让写记录中途断电、根没落的那次发布
/// 一份副本坏时每次可写挂载都拒、没有出口。施加前的点名单元验证照旧两份都要验过（[`every_named_unit_of_the_record_verifies`]），两处不共用。
pub(crate) fn every_named_unit_of_the_record_has_a_copy_that_verifies<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    record: &JournalRecord,
) -> bool {
    record.named.iter().all(|named| {
        let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
            return false;
        };
        named
            .locations
            .iter()
            .any(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))
    })
}

/// 点名项一条位置条目指的那一份读得出、整单元 CRC-32C 等于条目里的。
fn copy_named_by_the_location_entry_verifies<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    location: &LocationEntry,
    unit_bytes: usize,
) -> bool {
    reader
        .read(
            location.device,
            location.slot.to_device_offset(),
            unit_bytes,
        )
        .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
}

/// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
///
/// 链首锚在所选根覆盖的最后一条：所选根那次发布里带「本次发布末条」标志的那一条（已定项 14 注 1，读法乙；
/// 不取读得出的同 txg 记录里 jsn 最大的那条）。
/// 施加的单位是一次发布（第五条）：前四条判出来的前缀里，一次发布的记录要一直走到带末条标志（记录标志位 0）的那一条
/// 才整体施加；前缀停在一次发布中间（末条没到、断号、校验不过、下一条换了 txg、一次发布之内跳号、
/// 一个事务的提交标记没出现）⇒ 那次发布整体不施加。读者规则另外两格（D23（journal 的角色与格式） 已定项 4）同样断链、那次发布不施加：
/// 一次发布的首条序号不是 1（断在这一条）；带末条标志的那一条之后同一 (实例代号, checkpoint_txg) 里还有读得出的记录（断在带标志的那一条）。
///
/// # Errors
/// 所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写）⇒
/// `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`，一条记录都没施加；
/// 所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布没有 txg 可取）⇒ `UnitMalformed`
/// （[`CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`]），一条记录都没施加。
///
/// # Panics
/// `records` 的键不等于记录自己的 (实例代号, 计数器)，或有记录的计数器是 u64 的最大值：这两条是核的前置条件，调用方违反就是 bug。
pub fn replay_journal(
    reader: &dyn PoolReader,
    root: &RootRecord,
    ring_bytes: u64,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    named_unit_verification: NamedUnitVerification,
) -> Result<(JournalScanReport, RootRecord), RecoveryFailure> {
    let mut report = JournalScanReport {
        valid_records: records.len(),
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: 0,
    };
    // 取前缀那一截在 `singlefs_replay_prefix::replay_core` 里（Verus 验的是那一份）；这里只做三件：把记录表摊成核的入参、
    // 把两处读盘（反向链重读、点名单元校验）包成回调交给核、照核交回的结果拼报告与新根。
    let in_flight_limit =
        usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
    // 核的两条前置条件 Verus 只在核里假定、不在调用点查，在这里断言：
    for (key, record) in records {
        assert_eq!(
            *key,
            (record.instance, record.counter),
            "记录表的键要等于记录自己的 (实例代号, 计数器)：scan_journal 与各调用方都按它建键，键序因此就是核要的「按 (实例代号, 计数器) 严格升序」；\
             不等说明调用方建错了表，核的前置条件不成立"
        );
        assert!(
            record.counter < u64::MAX,
            "计数器是盘上 6 字节读来的（D23（journal 的角色与格式） 已定项 9 的 48 位），到不了 u64 的最大值；核要给链上下一号加一"
        );
    }
    let records_in_key_order: Vec<&JournalRecord> = records.values().collect();
    let core_records: Vec<CoreRecord<NewRootSource>> = records_in_key_order
        .iter()
        .enumerate()
        .map(|(index_in_key_order, record)| CoreRecord {
            instance: PrefixKernelInstanceGeneration(record.instance.0),
            counter: JournalCounter(record.counter),
            checkpoint_txg: PrefixKernelCheckpointTxg(record.checkpoint_txg.0),
            transaction: TransactionNumber(record.transaction),
            is_commit: record.is_commit,
            ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish(
                record.ordinal_within_publish.0,
            ),
            place_in_publish: match record.place_in_publish {
                JournalRecordPlaceInPublish::LastRecordOfThePublish => {
                    PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish
                }
                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {
                    PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                }
            },
            new_root_source: NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(
                index_in_key_order,
            ),
        })
        .collect();
    // 反向链（D23（journal 的角色与格式） 已定项 7「jsn 严格连续 + 校验和 + 反向链」；I-8.6（反向链算法））：
    // 校验和自洽、jsn 恰好接得上而链值不等的记录（上一条时间线的残留、改过的镜像）断在这一条，它所在的这次发布整体不施加。
    let back_chain_broken =
        |index: usize| match judge_back_chain_against_the_previous_record_on_disk(
            reader,
            records_in_key_order[index],
            ring_bytes,
            records,
        ) {
            BackChainJudgement::Holds | BackChainJudgement::PreviousRecordNotOnDisk => false,
            BackChainJudgement::Broken => true,
        };
    let named_units_verified =
        |index: usize| every_named_unit_of_the_record_verifies(reader, records_in_key_order[index]);
    let outcome = match replay_core(
        PrefixKernelInstanceGeneration(root.instance.0),
        PrefixKernelCheckpointTxg(root.checkpoint_txg.0),
        NewRootSource::ChosenRootItself,
        &core_records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
    ) {
        Ok(outcome) => outcome,
        // 核带出它看到的那几条计数器（按升序），直接拼报错。
        Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
            counters,
        }) => {
            return Err(
                RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                    counters: counters.iter().map(|counter| counter.0).collect(),
                },
            );
        }
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {
            return Err(RecoveryFailure::UnitMalformed {
                what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
            });
        }
    };
    report.above_water = outcome.report.above_water;
    report.prefix_applied = outcome.report.prefix_applied;
    report.verification_passed = outcome.report.verification_passed;
    report.verification_failed = outcome.report.verification_failed;
    report.maximum_applied_transaction = outcome.report.maximum_applied_transaction.0;
    let rebuilt = match outcome.root_source_after_replay {
        NewRootSource::ChosenRootItself => *root,
        NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(index_in_key_order) => {
            let record = records_in_key_order[index_in_key_order];
            // 每条记录都带整次发布的新根段（已定项 15），施加末条那一份：把所选根的九个字段换成记录里的——实例表指针、卸载记号、
            // 分配记录树根从新根段来，不再照抄所选根（用户 2026-10-02「回到甲」；`research/prompts/m3-ring-r3-main-verification.md`
            // 第二节第 1、2 行：照抄时首个文件、卸载两种发布施加出错根，专用清扫发布施加出旧实例表链）。fsid 新根段里没有，照所选根。
            RootRecord {
                filesystem_identifier: root.filesystem_identifier,
                unmount_marker: record.new_unmount_marker,
                instance: record.instance,
                checkpoint_txg: record.checkpoint_txg,
                tree_table: record.new_tree_table,
                tree_identifier_watermark: record.new_tree_identifier_watermark,
                rollback_floor: record.new_rollback_floor,
                instance_table: record.new_instance_table,
                mapping_root: record.new_mapping_root,
                allocation_record_tree_root: record.new_allocation_record_tree_root,
                // 回收门槛 t 与钉槽 p 原样带进拼出的根（C773（读坏与全零根槽的回收门槛钉只在内存里），用户 2026-10-20 定）。
                reclaim_threshold: record.new_reclaim_threshold,
                pinned_ring_slot: record.new_pinned_ring_slot,
            }
        }
    };
    Ok((report, rebuilt))
}

/// 一条记录的新根段拼出的那次发布的根：每条记录都带整次发布的新根段（D23（journal 的角色与格式） 已定项 15），九个字段取记录里的，
/// fsid 新根段里没有、取调用方给的。可写挂载读水位之上没施加的那次发布的账用它（`mount::placements_in_the_accounts_of_unapplied_publishes`）。
/// [`replay_journal`] 施加末条之后拼根是同一个拼法、在那里就地写着：那一段被变异表多行逐字锚着，这一批不挪它。
pub(crate) fn root_assembled_from_the_new_root_segment_of_the_record(
    record: &JournalRecord,
    filesystem_identifier: [u8; 16],
) -> RootRecord {
    RootRecord {
        filesystem_identifier,
        unmount_marker: record.new_unmount_marker,
        instance: record.instance,
        checkpoint_txg: record.checkpoint_txg,
        tree_table: record.new_tree_table,
        tree_identifier_watermark: record.new_tree_identifier_watermark,
        rollback_floor: record.new_rollback_floor,
        instance_table: record.new_instance_table,
        mapping_root: record.new_mapping_root,
        allocation_record_tree_root: record.new_allocation_record_tree_root,
        reclaim_threshold: record.new_reclaim_threshold,
        pinned_ring_slot: record.new_pinned_ring_slot,
    }
}

#[cfg(test)]
mod back_chain_judgement_tests {
    use super::{
        judge_back_chain_against_the_previous_record_on_disk, BackChainJudgement, PoolReader,
    };
    use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
    use crate::journal::{
        JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
    };
    use crate::pointer::NodePointer;
    use crate::root_record::{PinnedRingSlot, UnmountMarker};
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
    use std::collections::BTreeMap;

    /// 一块盘、读哪儿都读不回的读者：前一条在不在只看扫环交出来的那张表。
    struct PoolWhoseReadsReturnNothing;

    impl PoolReader for PoolWhoseReadsReturnNothing {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            vec![DeviceIdentity(0)]
        }
        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
            (device == DeviceIdentity(0)).then_some(4 << 30)
        }
        fn physical_block_size_probed_on_the_device(
            &self,
            _device: DeviceIdentity,
        ) -> Option<crate::block_device::PhysicalBlockSizeInBytes> {
            unreachable!("判前一条在不在只看扫环交出来的那张表，不探测块宽")
        }
        fn read(
            &self,
            _device: DeviceIdentity,
            _offset: DeviceOffsetInBytes,
            _length: usize,
        ) -> Option<Vec<u8>> {
            None
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            None
        }
    }

    fn record_of_instance_one_with_counter(counter: u64) -> JournalRecord {
        JournalRecord {
            instance: InstanceGeneration(1),
            counter,
            checkpoint_txg: CheckpointTxg(3),
            transaction: 1,
            is_commit: true,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(1),
            place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
            back_chain: 0,
            filesystem_identifier: 7,
            new_tree_table: NodePointer::empty_root(),
            new_mapping_root: NodePointer::empty_root(),
            new_tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
            new_rollback_floor: CheckpointTxg(0),
            new_instance_table: NodePointer::empty_root(),
            new_allocation_record_tree_root: NodePointer::empty_root(),
            new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            new_reclaim_threshold: CheckpointTxg(0),
            new_pinned_ring_slot: PinnedRingSlot::NO_SLOT_PINNED,
            named: Vec::new(),
        }
    }

    /// 计数器 0 没有记录（D23（journal 的角色与格式） 已定项 9 / 已定项 19：计数器从 1 起），盘上却可以有一条自证得过、计数器写成 0 的记录，
    /// 扫环照样把它收进表里（扫环按 (实例代号, 计数器) 建表，不看它落在环里哪一格）。判计数器 1 那一条的反向链时，前一条要是计数器 0，
    /// 不拿它当前一条：交回「前一条不在盘上」，不拿 0 去算它在环里的偏移（`journal::record_offset` 只收从 1 起的计数器）。
    /// 判别力：取前一条之前不判「计数器小 1 之后不小于 1」时找到的就是计数器 0 那一条，算偏移那一步 panic。
    #[test]
    fn record_of_counter_zero_in_the_scanned_ring_is_not_taken_as_the_previous_record_of_counter_one(
    ) {
        let records: BTreeMap<(InstanceGeneration, u64), JournalRecord> = [0, 1]
            .map(|counter| {
                (
                    (InstanceGeneration(1), counter),
                    record_of_instance_one_with_counter(counter),
                )
            })
            .into_iter()
            .collect();
        let record_of_counter_one = &records[&(InstanceGeneration(1), 1)];
        assert!(
            matches!(
                judge_back_chain_against_the_previous_record_on_disk(
                    &PoolWhoseReadsReturnNothing,
                    record_of_counter_one,
                    JOURNAL_RING_DEFAULT_BYTES,
                    &records,
                ),
                BackChainJudgement::PreviousRecordNotOnDisk
            ),
            "计数器 1 那一条的前一条不在盘上：计数器 0 那一条不是它的前一条"
        );
    }
}
