//! 录对拍用的流（新池新建文件、固定脚本到回退与它们改坏一处的几种），改坏单元之后重封校验和、沿引用链补到根记录。

use super::*;

pub(super) struct RecordedFlow {
    pub(super) name: &'static str,
    pub(super) base: MemoryPool,
    pub(super) writes: Vec<RetainedWrite>,
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) judged_root_index: usize,
    pub(super) versions: Vec<PublishedVersion>,
}

pub(super) fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

pub(super) fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

pub(super) fn record_new_pool_file_creation(tag: &str, name: &'static str) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        }],
    }
}

/// 改坏发布的根指着的树表单元（两份副本各一字节）：根落了的状态走读失败，CPU 与 GPU 都要红。
pub(super) fn record_new_pool_with_a_broken_tree_table(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-broken-tree-table");
    let publish_root = flow.judged_root_index;
    break_both_copies_of_the_tree_table_named_by_the_root_write(&mut flow, publish_root);
    flow
}

/// 第 `root_write` 次写（根槽写）指着的树表单元，两份副本各改坏末字节、不重封：那条根落了的状态上它的树表读不出。
pub(super) fn break_both_copies_of_the_tree_table_named_by_the_root_write(
    flow: &mut RecordedFlow,
    root_write: usize,
) {
    for copy in copies_of_the_tree_table_named_by_the_root_write(flow, root_write) {
        match &mut flow.writes[copy].contents {
            WrittenContents::Bytes(bytes) => {
                let last = bytes.len() - 1;
                bytes[last] ^= 0x01;
            }
            WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
        }
    }
}

/// 第 `root_write` 次写（根槽写）指着的树表单元的两份副本（写下标，位置条目 0、1 的次序）。
pub(super) fn copies_of_the_tree_table_named_by_the_root_write(
    flow: &RecordedFlow,
    root_write: usize,
) -> Vec<usize> {
    let root_bytes = flow.writes[root_write]
        .bytes()
        .expect("根槽写带字节")
        .to_vec();
    // 树表指针在根记录偏移 36：两条位置条目在 50 与 64（盘 4、槽 6、校验和 4）
    let locations: Vec<(u32, u64)> = [86usize, 100]
        .iter()
        .map(|offset| {
            let device =
                u32::from_le_bytes(root_bytes[*offset..*offset + 4].try_into().expect("4"));
            let mut slot = [0u8; 8];
            slot[..6].copy_from_slice(&root_bytes[*offset + 4..*offset + 10]);
            (device, u64::from_le_bytes(slot))
        })
        .collect();
    // 每条位置条目取根槽写之前落在那个槽上的最后一次单元写：那一份就是这条根指着的副本（同一个槽更早被别的单元用过时不碰那一次）
    let copies: Vec<usize> = locations
        .iter()
        .map(|(device, location_slot)| {
            flow.writes[..root_write]
                .iter()
                .rposition(|write| {
                    write.kind == StepKind::UnitWrite
                        && write.device.0 == *device
                        && write.offset.0 / 16384 == *location_slot
                })
                .expect("根槽写之前写过它指着的树表单元")
        })
        .collect();
    assert_ne!(copies[0], copies[1], "树表单元的两份副本是两次写");
    copies
}

/// 改一次写里的几个字节之后把那个结构的整槽校验和字段重封（CRC-32C 住字段前 4 字节、后 28 字节 0）：拦住它的只剩被改的字段本身。
pub(super) fn rewrite_and_reseal(
    flow: &mut RecordedFlow,
    kind: StepKind,
    nth_from_the_end: usize,
    cover_end: usize,
    field_offset: usize,
    rewrite: &dyn Fn(&mut Vec<u8>),
) {
    let index = flow
        .writes
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, write)| write.kind == kind)
        .nth(nth_from_the_end)
        .map(|(index, _)| index)
        .expect("流里有这一种写");
    match &mut flow.writes[index].contents {
        WrittenContents::Bytes(bytes) => {
            rewrite(bytes);
            let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
                bytes,
                cover_end,
                field_offset,
            );
            bytes[field_offset..field_offset + 32].copy_from_slice(&digest);
        }
        WrittenContents::Zeros { .. } => panic!("结构写是普通写"),
    }
}

/// 发布那条 journal 记录的提交标记字节改成 2（重封头校验和）：core 的读者当记录损坏，checker 的 I-8.8 判红。
pub(super) fn record_new_pool_with_a_commit_byte_of_two(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-journal-commit-byte-2");
    rewrite_and_reseal(&mut flow, StepKind::JournalRecord, 0, 4096, 46, &|bytes| {
        bytes[86] = 2;
    });
    flow
}

/// 发布那条 journal 记录的记录标志改成 0b11（重封）：core 拒收这条记录，checker 的 I-8.9 判「标志其余位」红。
pub(super) fn record_new_pool_with_record_flags_of_three(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-journal-flags-3");
    rewrite_and_reseal(&mut flow, StepKind::JournalRecord, 0, 4096, 46, &|bytes| {
        bytes[7] = 3;
    });
    flow
}

/// 最后一次系统配置槽写里固定结构槽距改成 512（重封）：那一槽自证过而带读者不收的值，core 整池拒（两遍恢复都没择到根），
/// checker 的 I-7.13 判红、别的不变量不判。
pub(super) fn record_new_pool_with_a_refused_slot_spacing(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-sysconfig-spacing-512");
    rewrite_and_reseal(
        &mut flow,
        StepKind::SystemConfigurationSlot,
        0,
        4096,
        155,
        &|bytes| {
            bytes[429..433].copy_from_slice(&512u32.to_le_bytes());
        },
    );
    flow
}

/// 发布那次根槽写之前，同一个根槽上垫几次写：垫满 32 次，发布的根落没落就只差组合掩码的高 32 位。
pub(super) const FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT: usize = 32;

/// 新池新建文件的流上，在暖机最后一次根槽写所在那一段之后另起一段：32 次根槽写，都写发布的根将要落的那个根槽、写的是它在基镜像里的
/// 字节（全零：那个槽在发布之前没有根）。这 32 次写不改任何一个崩溃态在盘上的字节（它们不带 journal 记录与单元，记录核对器也不看），
/// 只把发布那次根槽写挤成这个落点的第 33 次写：发布的根落了与没落的两个版本，组合掩码低 32 位相同（那 32 次都落了）、只差高位。
/// 判器查版本只比低 32 位时，把发布的根落了的状态读成全零的那一版，恢复落到暖机的根（C601（判器查版本不比组合掩码高 32 位时没有会红的流））。
pub(super) fn record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-publish-root-slot-past-32-writes");
    let publish_root = flow.writes[flow.judged_root_index].clone();
    let slot_bytes_in_the_base = singlefs_core::recovery::PoolReader::read(
        &flow.base,
        publish_root.device,
        publish_root.offset,
        usize::try_from(publish_root.length_in_bytes()).expect("根槽写的长度装得进 usize"),
    )
    .expect("基镜像里读得到发布的根要落的槽");
    assert!(
        slot_bytes_in_the_base.iter().all(|byte| *byte == 0),
        "发布的根落在一个基镜像里空着的根槽上（垫的写写全零，不改任何崩溃态的盘上字节）"
    );
    assert!(
        flow.writes[..flow.judged_root_index]
            .iter()
            .all(|write| (write.device, write.offset) != (publish_root.device, publish_root.offset)),
        "发布之前流里没有别的写落在这个根槽上"
    );
    let warm_up_root = flow.writes[..flow.judged_root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布之前有暖机的根槽写");
    let segment_of_the_warm_up_root = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&warm_up_root))
        .expect("暖机的根槽写在某一段里");
    let first_filler = *flow.segments[segment_of_the_warm_up_root]
        .last()
        .expect("段里至少一次写")
        + 1;
    let filler = RetainedWrite {
        device: publish_root.device,
        kind: StepKind::RootRecordFua,
        is_force_unit_access: publish_root.is_force_unit_access,
        offset: publish_root.offset,
        contents: WrittenContents::Bytes(slot_bytes_in_the_base),
    };
    flow.writes.splice(
        first_filler..first_filler,
        vec![filler; FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT],
    );
    for segment in &mut flow.segments {
        for write_index in segment.iter_mut() {
            if *write_index >= first_filler {
                *write_index += FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT;
            }
        }
    }
    flow.segments.insert(
        segment_of_the_warm_up_root + 1,
        (first_filler..first_filler + FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT).collect(),
    );
    flow.judged_root_index += FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT;
    flow
}

pub(super) fn record_fixed_script_through_rollback(tag: &str) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    let second = overwrite_step(
        &mut recorder,
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(4),
        content: second_content(),
    });
    pool.output = second;
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in 5..=7 {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: second_content(),
        });
    }
    let third = overwrite_step(
        &mut recorder,
        &mut pool,
        &third_content(),
        InstanceGeneration(2),
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(8),
        content: third_content(),
    });
    pool.output = third;
    roll_back_step(
        &mut recorder,
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(9),
        content: file_content(),
    });
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedFlow {
        name: "fixed-script-through-rollback",
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

pub(super) fn crc32c(bytes: &[u8]) -> u32 {
    singlefs_core::checksum::crc32_castagnoli(bytes)
}

/// 一个单元重封自己的两道校验和：载荷 CRC（按类的偏移，罩明文头之后到单元末）与头校验和（字段 10，罩明文头）。
pub(super) fn reseal_both_checksums_of_the_unit_bytes(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105, 101),
        2 => {
            let key_width = usize::from(bytes[51]);
            (86 + 2 * key_width, 76 + 2 * key_width)
        }
        3 => (107, 89),
        other => panic!("单元类标签 {other} 不在 1 / 2 / 3 里"),
    };
    let payload_crc = crc32c(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, header_end, 10);
    bytes[10..42].copy_from_slice(&digest);
}

/// journal 记录重封：载荷校验和（95，罩点名项）再整条头校验和（46，罩 4096）。
pub(super) fn reseal_journal_record(bytes: &mut [u8]) {
    let named = usize::try_from(u32::from_le_bytes(bytes[12..16].try_into().expect("4")))
        .expect("点名项数");
    let payload_crc = crc32c(
        &bytes[JOURNAL_RECORD_HEADER_BYTES
            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
    );
    bytes[95..99].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, 4096, 46);
    bytes[46..78].copy_from_slice(&digest);
}

/// 下一条记录该带的反向链：这一条 496 字节头、校验和字段按 0 参与的 CRC-32C。
pub(super) fn journal_chain_value(bytes: &[u8]) -> u32 {
    let mut header = bytes[..JOURNAL_RECORD_HEADER_BYTES].to_vec();
    header[46..78].fill(0);
    crc32c(&header)
}

pub(super) fn reseal_root_record(bytes: &mut [u8]) {
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, bytes.len(), 138);
    bytes[138..170].copy_from_slice(&digest);
}

/// 改坏一批单元写（同一个单元的两份副本，或只改其中一份）之后把校验和一路重封到根：被改的单元先重封自己的两道校验和，
/// 再把每一处指着它的位置条目（树节点、根记录、journal 点名项里的 盘 4 + 槽 6 + 整单元校验和 4）换成新的整单元 CRC，
/// 被改的那些结构再各自重封（单元继续往上传；journal 记录还要把同实例下一条的反向链改成它新的链值、一路传下去）。
/// 这样留下的坏法只剩被改的那个字段本身：CPU 的 checker 红在那个字段管的不变量上，GPU 要红在同样的状态、同样的项上。
pub(super) fn corrupt_units_and_reseal_up_to_the_root(
    flow: &mut RecordedFlow,
    targets: &[usize],
    mutate: &dyn Fn(&mut [u8]),
) {
    corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
        flow,
        targets,
        mutate,
        OwnChecksumsOfTheChangedUnits::Resealed,
    );
}

/// 被改的单元自己那两道校验和（载荷 CRC、头校验和）重不重封。
#[derive(Clone, Copy, Debug)]
pub(super) enum OwnChecksumsOfTheChangedUnits {
    /// 重封：被改的单元自证得过，坏法只剩改的那个字段。
    Resealed,
    /// 不重封：被改的单元按自己的类认不出、或自证不过（类标签改成不登记的码那一形），指着它的位置条目照样换成新的整单元 CRC。
    LeftAsBeforeTheChange,
}

/// 同 [`corrupt_units_and_reseal_up_to_the_root`]，被改的单元自己那两道校验和照 `own_checksums` 重封或不动；
/// 指着它的位置条目一律换成改后的整单元 CRC、沿引用链一路重封到根。
pub(super) fn corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
    flow: &mut RecordedFlow,
    targets: &[usize],
    mutate: &dyn Fn(&mut [u8]),
    own_checksums: OwnChecksumsOfTheChangedUnits,
) {
    let mut pending: Vec<(usize, u32)> = Vec::new();
    for target in targets {
        let write = &mut flow.writes[*target];
        assert_eq!(write.kind, StepKind::UnitWrite, "改的是单元写");
        let WrittenContents::Bytes(bytes) = &mut write.contents else {
            panic!("单元写是普通写");
        };
        let old_crc = crc32c(bytes);
        mutate(bytes);
        match own_checksums {
            OwnChecksumsOfTheChangedUnits::Resealed => {
                reseal_both_checksums_of_the_unit_bytes(bytes)
            }
            OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange => {}
        }
        pending.push((*target, old_crc));
    }
    while let Some((index, old_crc)) = pending.pop() {
        let (device, slot, new_crc) = {
            let write = &flow.writes[index];
            let bytes = write.bytes().expect("单元写带字节");
            (write.device.0, write.offset.0 / 16384, crc32c(bytes))
        };
        if old_crc == new_crc {
            continue;
        }
        let mut pattern = Vec::with_capacity(14);
        pattern.extend_from_slice(&device.to_le_bytes());
        pattern.extend_from_slice(&slot.to_le_bytes()[..6]);
        pattern.extend_from_slice(&old_crc.to_le_bytes());
        let mut journal_records_resealed: Vec<usize> = Vec::new();
        for referrer in 0..flow.writes.len() {
            let kind = flow.writes[referrer].kind;
            let WrittenContents::Bytes(bytes) = &mut flow.writes[referrer].contents else {
                continue;
            };
            let mut replaced = false;
            let mut position = 0;
            while position + 14 <= bytes.len() {
                if bytes[position..position + 14] == pattern[..] {
                    bytes[position + 10..position + 14].copy_from_slice(&new_crc.to_le_bytes());
                    replaced = true;
                    position += 14;
                } else {
                    position += 1;
                }
            }
            if !replaced {
                continue;
            }
            match kind {
                StepKind::UnitWrite => {
                    let referrer_old_crc = {
                        // 位置条目已换、校验和还没重封：整单元 CRC 要按重封之前的原样算，用写进去之前的字节复原
                        let mut before = bytes.clone();
                        let mut restore_position = 0;
                        while restore_position + 14 <= before.len() {
                            if before[restore_position..restore_position + 10] == pattern[..10]
                                && before[restore_position + 10..restore_position + 14]
                                    == new_crc.to_le_bytes()
                            {
                                before[restore_position + 10..restore_position + 14]
                                    .copy_from_slice(&old_crc.to_le_bytes());
                                restore_position += 14;
                            } else {
                                restore_position += 1;
                            }
                        }
                        crc32c(&before)
                    };
                    reseal_both_checksums_of_the_unit_bytes(bytes);
                    pending.push((referrer, referrer_old_crc));
                }
                StepKind::RootRecordFua => reseal_root_record(bytes),
                StepKind::JournalRecord => {
                    reseal_journal_record(bytes);
                    journal_records_resealed.push(referrer);
                }
                StepKind::ZeroFill
                | StepKind::Discard
                | StepKind::SystemConfigurationSlot
                | StepKind::Barrier => {
                    panic!("位置条目不该出现在 {kind:?} 里")
                }
            }
        }
        carry_the_back_chain_forward_from(flow, journal_records_resealed);
    }
}

/// 反向链一路传下去：这几条 journal 记录写重封过，同盘、同实例、计数器大 1 的那一条带的是前一条头的链值——不等就改成它、重封，再往下传。
pub(super) fn carry_the_back_chain_forward_from(
    flow: &mut RecordedFlow,
    mut journal_records_resealed: Vec<usize>,
) {
    while let Some(record_index) = journal_records_resealed.pop() {
        let (record_device, instance, counter, chain) = {
            let write = &flow.writes[record_index];
            let bytes = write.bytes().expect("记录写带字节");
            let mut counter = [0u8; 8];
            counter[..6].copy_from_slice(&bytes[20..26]);
            (
                write.device,
                u32::from_le_bytes(bytes[16..20].try_into().expect("4")),
                u64::from_le_bytes(counter),
                journal_chain_value(bytes),
            )
        };
        for next in 0..flow.writes.len() {
            if flow.writes[next].kind != StepKind::JournalRecord
                || flow.writes[next].device != record_device
            {
                continue;
            }
            let WrittenContents::Bytes(bytes) = &mut flow.writes[next].contents else {
                continue;
            };
            let mut next_counter = [0u8; 8];
            next_counter[..6].copy_from_slice(&bytes[20..26]);
            if u32::from_le_bytes(bytes[16..20].try_into().expect("4")) != instance
                || u64::from_le_bytes(next_counter) != counter + 1
                || bytes[91..95] == chain.to_le_bytes()
            {
                continue;
            }
            bytes[91..95].copy_from_slice(&chain.to_le_bytes());
            reseal_journal_record(bytes);
            journal_records_resealed.push(next);
        }
    }
}

/// 流里最后一个单元（两份副本：最后两条命中 `select` 的单元写，同槽同字节）的写下标；`only_first_copy` 只取盘序小的那一份。
pub(super) fn last_unit_copies(
    flow: &RecordedFlow,
    what: &str,
    select: &dyn Fn(&[u8]) -> bool,
    only_first_copy: bool,
) -> Vec<usize> {
    let matching: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite && write.bytes().is_some_and(select))
        .map(|(index, _)| index)
        .collect();
    assert!(
        matching.len() >= 2,
        "流 {} 里命中「{what}」的单元写只有 {} 条，要至少两条（一个单元的两份副本）；流里的单元写按类 / key 宽 / 条目宽 / 记录类型：{:?}",
        flow.name,
        matching.len(),
        flow.writes
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .filter_map(|write| write.bytes())
            .map(|bytes| match unit_class(bytes) {
                2 => format!("2/k{}/w{}", index_node_key_width(bytes), index_node_entry_width(bytes)),
                3 => format!("3/t{}", packed_record_type(bytes)),
                other => format!("{other}"),
            })
            .collect::<Vec<_>>()
    );
    let copies = &matching[matching.len() - 2..];
    assert_eq!(
        flow.writes[copies[0]].offset, flow.writes[copies[1]].offset,
        "最后两条命中的单元写是同一个单元的两份副本（同槽）"
    );
    assert_eq!(
        flow.writes[copies[0]].bytes(),
        flow.writes[copies[1]].bytes(),
        "两份副本字节相同"
    );
    if only_first_copy {
        let first = if flow.writes[copies[0]].device.0 < flow.writes[copies[1]].device.0 {
            copies[0]
        } else {
            copies[1]
        };
        vec![first]
    } else {
        copies.to_vec()
    }
}

pub(super) fn unit_class(bytes: &[u8]) -> u8 {
    bytes[6]
}

pub(super) fn index_node_key_width(bytes: &[u8]) -> u8 {
    bytes[51]
}

pub(super) fn index_node_entry_width(bytes: &[u8]) -> u16 {
    let header_end = 86 + 2 * usize::from(bytes[51]);
    u16::from_le_bytes(bytes[header_end - 2..header_end].try_into().expect("2"))
}

pub(super) fn packed_record_type(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes[51..53].try_into().expect("2"))
}

/// 新池新建文件的流上改坏最后一个 `select` 命中的单元（两份或只一份）的一个字段、重封到根：`name` 是流名，`invariant` 是该红的那条。
pub(super) fn record_new_pool_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    only_first_copy: bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    let targets = last_unit_copies(&flow, name, select, only_first_copy);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// 固定脚本到回退 D 的流上同样改坏最后一个命中的单元。
pub(super) fn record_rollback_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = name;
    let targets = last_unit_copies(&flow, name, select, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// 改坏一个字段、重封到根的几个必红的世界：被改的字段与它管的不变量（每个世界一条流、一条用例）。
#[derive(Clone, Copy, Debug)]
pub(super) enum ResealedFieldWorld {
    /// inode 记录的 blocks 字段加 1（I-9.15）。
    InodeBlocksPlusOne,
    /// 数据单元五元组里的对象出生代加 1（I-9.10）。
    DataUnitObjectBirthPlusOne,
    /// 分配记录叶第一条记录的分配代加 1（I-3.10）。
    AllocationGenerationPlusOne,
    /// 分配记录叶第二条记录的槽号改成第一条的（I-5.4）。
    AllocationRecordsOverlap,
    /// 记账叶第一条的值加 1（I-3.1）。
    AccountingValuePlusOne,
    /// 树表前两条条目的树 ID 互换（I-9.16）。
    TreeTableOutOfOrder,
    /// 映射叶第一条 key 的出生 txg 加 1（I-1.11）。
    MappingKeyBirthPlusOne,
    /// 回退流上最后一版实例表第一行的实例代号加 1（I-3.8）。
    RollbackInstanceRowNotBelowTheMount,
    /// 数据单元只改一份副本的载荷首字节（I-1.8）。
    DataUnitCopiesDiffer,
    /// 回退流上最后一版树表里 inode 树条目的诞生 txg 加 1（I-9.14）。
    RollbackTreeTableBirthTxgPlusOne,
    /// 回退流上最后一版分配记录叶里带已释放标志的第一条记录的释放代加 8（I-3.9）。
    RollbackReleaseGenerationPlusEight,
}

/// 录下这一个世界的流，交回它与 CPU 池级 checker 要在它上面红的那一条。
pub(super) fn record_the_resealed_field_world(
    world: ResealedFieldWorld,
) -> (RecordedFlow, &'static str) {
    let inode_leaf = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 2;
    let instance_table = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 4;
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let allocation_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 10
            && index_node_entry_width(bytes) == 20
    };
    // 带至少一条已释放记录（跨度段位 15）的分配记录叶
    let allocation_leaf_with_a_released_record = |bytes: &[u8]| {
        if !allocation_leaf(bytes) {
            return false;
        }
        let header_end = 86 + 2 * usize::from(bytes[51]);
        let count = usize::from(u16::from_le_bytes(
            bytes[header_end - 4..header_end - 2].try_into().expect("2"),
        ));
        (0..count).any(|record| bytes[135 + record * 20 + 11] & 0x80 != 0)
    };
    let accounting_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 22
            && index_node_entry_width(bytes) == 34
    };
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    let mapping_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2 && index_node_key_width(bytes) == 27 && bytes[50] == 0
    };
    match world {
        ResealedFieldWorld::InodeBlocksPlusOne => (
            // inode 记录的 blocks 字段加 1：blocks ≠ ⌈size ÷ 512⌉
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i915",
                "new-pool-inode-blocks-plus-one",
                &inode_leaf,
                false,
                &|bytes| {
                    bytes[136 + 48] = bytes[136 + 48].wrapping_add(1);
                },
            ),
            "I-9.15",
        ),
        ResealedFieldWorld::DataUnitObjectBirthPlusOne => (
            // 数据单元五元组里的对象出生代加 1：与 inode 记录的对象出生代不符
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i910",
                "new-pool-data-unit-object-birth-plus-one",
                &data_unit,
                false,
                &|bytes| {
                    bytes[59] = bytes[59].wrapping_add(1);
                },
            ),
            "I-9.10",
        ),
        ResealedFieldWorld::AllocationGenerationPlusOne => (
            // 分配记录叶第一条记录的分配代加 1：分配代 ≠ 它罩住的单元头里的诞生代号
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i310",
                "new-pool-allocation-generation-plus-one",
                &allocation_leaf,
                false,
                &|bytes| {
                    bytes[135 + 12] = bytes[135 + 12].wrapping_add(1);
                },
            ),
            "I-3.10",
        ),
        ResealedFieldWorld::AllocationRecordsOverlap => (
            // 分配记录叶第二条记录的槽号改成第一条的：两条记录罩住同一个槽
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i54",
                "new-pool-allocation-records-overlap",
                &allocation_leaf,
                false,
                &|bytes| {
                    let first = bytes[135..135 + 10].to_vec();
                    bytes[135 + 20..135 + 30].copy_from_slice(&first);
                },
            ),
            "I-5.4",
        ),
        ResealedFieldWorld::AccountingValuePlusOne => (
            // 记账叶第一条的值加 1：已分配 / 空闲 / defer 的账对不上遍历
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i31",
                "new-pool-accounting-value-plus-one",
                &accounting_leaf,
                false,
                &|bytes| {
                    bytes[159 + 22] = bytes[159 + 22].wrapping_add(1);
                },
            ),
            "I-3.1",
        ),
        ResealedFieldWorld::TreeTableOutOfOrder => (
            // 树表前两条条目的树 ID 互换：不按树 ID 严格升序
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i916",
                "new-pool-tree-table-out-of-order",
                &tree_table,
                false,
                &|bytes| {
                    let first = bytes[131..131 + 8].to_vec();
                    let second = bytes[131 + 200..131 + 208].to_vec();
                    bytes[131..131 + 8].copy_from_slice(&second);
                    bytes[131 + 200..131 + 208].copy_from_slice(&first);
                },
            ),
            "I-9.16",
        ),
        ResealedFieldWorld::MappingKeyBirthPlusOne => (
            // 映射叶第一条 key 的出生 txg 加 1：映射 key 与单元头不符
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i111",
                "new-pool-mapping-key-birth-plus-one",
                &mapping_leaf,
                false,
                &|bytes| {
                    bytes[169 + 9] = bytes[169 + 9].wrapping_add(1);
                },
            ),
            "I-1.11",
        ),
        ResealedFieldWorld::RollbackInstanceRowNotBelowTheMount => (
            // 回退流上最后一版实例表第一行的实例代号加 1：行不再低于挂载根的实例（新池流没有实例表写：取号那次写在 mkfs 那一段里）
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i38",
                "rollback-instance-row-not-below-mount",
                &instance_table,
                &|bytes| {
                    bytes[136 + 1] = bytes[136 + 1].wrapping_add(1);
                },
            ),
            "I-3.8",
        ),
        ResealedFieldWorld::DataUnitCopiesDiffer => (
            // 数据单元只改一份副本的载荷首字节：归并成一组的两份载荷不同
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i18",
                "new-pool-data-unit-copies-differ",
                &data_unit,
                true,
                &|bytes| {
                    bytes[134] ^= 0xFF;
                },
            ),
            "I-1.8",
        ),
        ResealedFieldWorld::RollbackTreeTableBirthTxgPlusOne => (
            // 回退流上最后一版树表里 inode 树条目的诞生 txg 加 1：跨根不一
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i914",
                "rollback-tree-table-birth-txg-plus-one",
                &tree_table,
                &|bytes| {
                    for entry in 0..(bytes.len() - 131) / 200 {
                        let offset = 131 + entry * 200;
                        if u16::from_le_bytes(
                            bytes[offset + 10..offset + 12].try_into().expect("2"),
                        ) == 2
                        {
                            bytes[offset + 108] = bytes[offset + 108].wrapping_add(1);
                        }
                    }
                },
            ),
            "I-9.14",
        ),
        ResealedFieldWorld::RollbackReleaseGenerationPlusEight => (
            // 回退流上最后一版分配记录叶里带已释放标志的第一条记录的释放代加 8：落到区间之外
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i39",
                "rollback-release-generation-plus-eight",
                &allocation_leaf_with_a_released_record,
                &|bytes| {
                    let header_end = 86 + 2 * usize::from(bytes[51]);
                    let count = usize::from(u16::from_le_bytes(
                        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
                    ));
                    for record in 0..count {
                        let offset = 135 + record * 20;
                        if bytes[offset + 11] & 0x80 != 0 {
                            bytes[offset + 12] = bytes[offset + 12].wrapping_add(8);
                            break;
                        }
                    }
                },
            ),
            "I-3.9",
        ),
    }
}
