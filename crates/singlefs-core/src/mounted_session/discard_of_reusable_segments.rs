//! 发布返回之后对整段可再分配的聚簇段发 discard（[`discard_the_segments_reusable_as_a_whole`]），装着还没清扫的孤儿头的段不发。

use super::*;

/// 发布返回之后发 discard（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G5，判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
/// 第二节 S6）：「可再分配」谓词为真（已回收、没被抬 F 扣住、没被隔离、F 已生效）的槽按 64 槽对齐的整段发——只发回收过槽、整段可再分配的段
/// （`allocator::PoolAllocator::take_the_segments_to_discard_on_every_device`），每块盘发它自己那几段。discard 取几态、设备日志怎么判不在这里
/// （归设备侧与崩溃点重放）。长度恒为一段（64 × 16384 字节），不发长度 0 的。
///
/// 装着还没清扫的孤儿头的段不发（主 agent 2026-10-15 定，零轮）：discard 的结局是三态之一（D13（验证路线） 已定项 4），不能拿它当擦除——
/// 抹孤儿头归实例表清扫（`instance_table_sweep`），清扫判过、抹过之前段里的孤儿头要原样留着。孤儿头 = 段里某个槽开头认得出本池的单元头
/// （清扫认头的那一判，`instance_table_sweep::unit_head_read_by_sweep`），它的写序实例在现行那一版的实例表里还有一行、按那一行判未发布
/// （`UnitHeadReadBySweep::is_unpublished_by_the_row`）。判之前读不出实例表、读不出段里的一个槽，这一段同样不发（读不到不等于没有）。
///
/// 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）时第一道就交回、一段都不发，待 discard 表不取、原样留着
/// （C928（冻结着时整理照发 discard），判决 `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R3 K3-1 两个候选里取「discard 开头判冻结」）：
/// 冻结时分配器退回发布之前那一份，退回的那一份里整段可再分配的段可能正装着冻结那次已落盘的单元。判在这里而不在整理入口：发 discard 的三处
/// （用户改动发成之后、整理显式入口、后台入口）一处罩住，整理入口的错误成员表不动；整理那一趟自己的发布本来就在发布路径第一道被冻结拒。
pub fn discard_the_segments_reusable_as_a_whole<Device: BlockDevice>(
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    parameters: &MakeFilesystemParameters,
    current_root: &RootRecord,
) -> DiscardsAfterThePublishes {
    let mut discards = DiscardsAfterThePublishes::default();
    if let Some(frozen) = allocator.frozen_publish() {
        discards.txg_of_the_frozen_publish_withholding_every_discard =
            Some(frozen.checkpoint_txg());
        return discards;
    }
    let segments_to_discard = allocator.take_the_segments_to_discard_on_every_device();
    if segments_to_discard
        .iter()
        .all(|(_, segment_starts)| segment_starts.is_empty())
    {
        return discards;
    }
    // 经数只坏一份的那一层读（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：下一次用户改动发成之后另发的那一批换下它。
    let devices_counting_unit_reads_with_one_copy_failing =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
    let rows_of_the_instance_table: Option<BTreeMap<InstanceGeneration, InstanceRow>> =
        instance_table_chain_of_root(
            &devices_counting_unit_reads_with_one_copy_failing,
            current_root,
        )
        .ok()
        .map(|chain| {
            chain
                .records
                .rows
                .iter()
                .map(|row| (row.instance, *row))
                .collect()
        });
    allocator.count_the_other_unit_reads_with_one_copy_failing(
        devices_counting_unit_reads_with_one_copy_failing
            .locations_of_the_units_read_with_one_copy_failing(),
    );
    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    for (device_identity, segment_starts) in segments_to_discard {
        for segment_start in segment_starts {
            let holding = match &rows_of_the_instance_table {
                Some(rows) => segment_holding_unswept_orphan_heads(
                    &*devices,
                    device_identity,
                    segment_start,
                    rows,
                    filesystem_identifier,
                ),
                None => SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed,
            };
            match holding {
                SegmentHoldingUnsweptOrphanHeads::No => {}
                SegmentHoldingUnsweptOrphanHeads::Yes => {
                    discards
                        .withheld_holding_unswept_orphan_heads
                        .push((device_identity, segment_start));
                    continue;
                }
                SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed => {
                    discards
                        .withheld_after_a_failed_read
                        .push((device_identity, segment_start));
                    continue;
                }
            }
            let Some((_, device)) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device_identity)
            else {
                continue;
            };
            match device.discard_at(
                segment_start.to_device_offset(),
                CLUSTER_SEGMENT_SLOTS * SLOT_BYTES,
            ) {
                Ok(DiscardOutcome::Applied) => {
                    discards.applied.push((device_identity, segment_start));
                }
                Ok(DiscardOutcome::Unsupported) => {
                    discards.unsupported.push((device_identity, segment_start));
                }
                Err(_cause) => discards.failed.push((device_identity, segment_start)),
            }
        }
    }
    discards
}

/// 一段装没装着还没清扫的孤儿头（[`discard_the_segments_reusable_as_a_whole`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SegmentHoldingUnsweptOrphanHeads {
    No,
    Yes,
    /// 段里一个槽读不出：判不了，按装着算（不发）。
    NotJudgedReadFailed,
}

/// `device` 上从 `segment_start` 起的那一段：逐槽读一整槽，开头认得出本池的单元头、写序实例在 `rows` 里有一行、按那一行判未发布的，就是还没清扫的孤儿头。
/// 认出一个头就按它的整单元跨度跳（同清扫的认领规则）。迭代上界是一段 64 槽；提前出口是认出一个孤儿头与读失败。
fn segment_holding_unswept_orphan_heads<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    segment_start: SlotNumber,
    rows: &BTreeMap<InstanceGeneration, InstanceRow>,
    filesystem_identifier: u64,
) -> SegmentHoldingUnsweptOrphanHeads {
    let slot_bytes = usize::try_from(SLOT_BYTES).expect("一槽 16384 字节");
    let mut slot = segment_start.0;
    while slot < segment_start.0 + CLUSTER_SEGMENT_SLOTS {
        let Some(bytes) = reader.read(device, SlotNumber(slot).to_device_offset(), slot_bytes)
        else {
            return SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed;
        };
        let Some(head) = unit_head_read_by_sweep(&bytes) else {
            slot += 1;
            continue;
        };
        slot += head.span_in_slots();
        if head.filesystem_identifier == filesystem_identifier
            && rows
                .get(&head.writer_instance)
                .is_some_and(|row| head.is_unpublished_by_the_row(row))
        {
            return SegmentHoldingUnsweptOrphanHeads::Yes;
        }
    }
    SegmentHoldingUnsweptOrphanHeads::No
}
