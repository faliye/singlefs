
// ───────────── 攻方原型（m3-batch59-code-r1 opus；rerun.sh 只接在仓副本这份文件的末尾）：L4 那一条换几种造法 ─────────────

/// 这一版（覆盖写那一版，txg 4）自己那行水位怎么弄没。
#[derive(Clone, Copy, Debug)]
enum OpusL4Construction {
    /// 被判那一份（第 59 批 L4）：删自己那行，补上一版（txg 3）盘 0 的已分配字节 (1, 0, 0, 3)。
    PreviousVersionsAllocatedBytesRow,
    /// 第 57 批那一份：自己那行的统计量标签改成 13，别的段不动。
    OwnRowRelabelledThirteen,
    /// 我提的：删自己那行，补一行登记过、这一版没写的统计量 7（扩展点配额已用量，D5 已定项 4 表里不带设备维、不带树维），
    /// 树段 0、设备段 0xFFFF_FFFF（D5 已定项 10 的两个保留值），代段 = 这一版的 txg，seq 照自己那行。
    RegisteredStatisticSevenOfThisVersion,
}

/// 照被判那一条的步骤造盘（同一个池、同一个改记账行的写法、同一个根槽原地换），交回从盘上重建 txg 4 那一版与可写挂载各报什么，
/// 另交回改过的记账行（按 key 排）。
fn opus_probe_b59_l4_outcomes(
    construction: OpusL4Construction,
) -> (Option<RebuildVersionFailure>, Option<MountError>, Vec<AccountingEntry>) {
    let (mut pool, overwrites) =
        pool_after_the_first_file_and_the_overwrites(&[content_of(2000, 1)]);
    let overwrite = &overwrites[0];
    let overwrite_txg = overwrite.root.checkpoint_txg;
    let previous_versions_row = *pool
        .first
        .accounting_entries
        .iter()
        .find(|row| row.statistic == STATISTIC_ALLOCATED_BYTES && row.device == DeviceIdentity(0))
        .expect("第一个文件那一版每盘一行已分配字节");
    let rows_after = std::cell::RefCell::new(Vec::new());
    let damaged_root =
        root_whose_accounting_rows_are_rewritten(&mut pool.image, &overwrite.root, |rows| {
            let own_row = rows.pop().expect("记账树恒有池级 3 行");
            assert_eq!(
                (own_row.statistic, own_row.generation),
                (STATISTIC_INODE_WATERMARK, overwrite_txg),
                "末行是这一版自己那一行水位"
            );
            rows.push(match construction {
                OpusL4Construction::PreviousVersionsAllocatedBytesRow => previous_versions_row,
                OpusL4Construction::OwnRowRelabelledThirteen => AccountingEntry {
                    statistic: STATISTIC_INODE_WATERMARK + 1,
                    ..own_row
                },
                OpusL4Construction::RegisteredStatisticSevenOfThisVersion => AccountingEntry {
                    statistic: 7,
                    tree: singlefs_core::address::TreeIdentifier(0),
                    device: DeviceIdentity(singlefs_core::records::STATISTIC_NO_DEVICE_DIMENSION),
                    generation: overwrite_txg,
                    value: 0,
                    sequence: own_row.sequence,
                },
            });
            rows.sort_by_key(AccountingEntry::sort_key);
            *rows_after.borrow_mut() = rows.clone();
        });
    let ring_slot = target_for_publish(overwrite_txg, pool.geometry.root_ring_slots_per_region);
    let device_holding_the_root = DeviceIdentity(
        ROOT_RING_REGION_DEVICES[usize::try_from(ring_slot.region).expect("区域号小于 3")],
    );
    pool.image
        .devices
        .get_mut(&device_holding_the_root)
        .expect("区域落在池里的盘上")
        .write(
            slot_offset(ring_slot, pool.geometry.fixed_structure_slot_spacing),
            &damaged_root
                .to_slot(usize::try_from(pool.geometry.physical_block_size).expect("根槽宽")),
        );
    let rebuilt = rebuild_version(&pool.image, &damaged_root, Some(overwrite.record.clone())).err();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
        .image
        .devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE);
            device.image = image.clone();
            (*identity, device)
        })
        .collect();
    let mounted = mount_writable(
        &parameters_with(geometry_with_minimum_input_output_bytes(512)),
        &mut devices,
    )
    .err();
    (rebuilt, mounted, rows_after.into_inner())
}

/// 只打印：每种造法下重建与可写挂载各报什么，改过的这一版里统计量 12 的行与补进来的那一行。
#[test]
fn opus_probe_b59_l4_constructions() {
    for construction in [
        OpusL4Construction::PreviousVersionsAllocatedBytesRow,
        OpusL4Construction::OwnRowRelabelledThirteen,
        OpusL4Construction::RegisteredStatisticSevenOfThisVersion,
    ] {
        let (rebuilt, mounted, rows) = opus_probe_b59_l4_outcomes(construction);
        let keys: Vec<(u16, u64, u32, u64)> = rows.iter().map(AccountingEntry::sort_key).collect();
        println!(
            "OPUS_M3 l4 construction={construction:?} rows={} keys={keys:?} rebuild={rebuilt:?} writable={mounted:?}",
            rows.len()
        );
    }
}

/// 我提的造法：重建与可写挂载都报水位行缺（与被判那一条同一个断言）。
#[test]
fn opus_probe_b59_l4_registered_statistic_seven_reaches_the_watermark_judge() {
    let (rebuilt, mounted, _rows) =
        opus_probe_b59_l4_outcomes(OpusL4Construction::RegisteredStatisticSevenOfThisVersion);
    assert_eq!(
        rebuilt,
        Some(RebuildVersionFailure::Walk(
            RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree
        )),
        "统计量 7 那一造法：重建报水位行缺"
    );
    assert!(
        matches!(
            mounted,
            Some(MountError::Recovery(
                RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree
            ))
        ),
        "统计量 7 那一造法：可写挂载报水位行缺：{mounted:?}"
    );
}
