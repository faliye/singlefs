//! checker 档模块：crash
//! 代码审阅第 4 条（这一批只量、不实现）：层 0 把一次写的撕裂态并进「没持久」，对原地覆写不成立——原地覆写撕裂时，那一处的旧内容
//! 也没了，而「没持久」留着完整的旧内容。用户 2026-09-27 定「原地覆写补第三态」（新旧都读不出），状态数先量：
//! 这里按段序列算「原地覆写的写取 3 态（没持久 / 持久 / 新旧都读不出）、其余写取 2 态」时全量与甲二快档的闭式，与今天的并排。
//!
//! 原地覆写的写怎么认（[`is_tearable_in_place_overwrite`]），三条都成立才算：
//! 1. 不是单元写：单元写是 COW，落在分配器交出来的槽上，那一槽的旧内容没有谁还要（回收谓词管着），撕裂与没持久一样，照旧 2 态；
//! 2. 长于一个扇区：一个扇区的写撕不开（层 0 的撕裂粒度就是扇区），根槽写是一个物理块，照旧 2 态；
//! 3. 它罩住的范围里原来有东西：基镜像那一段有非零字节，或写表里更早有一次同一块盘上、不是整段清零的写与它重叠。
//!
//! 一段 n 个写、其中 m 个取 3 态：全量 3^m · 2^(n−m) − 1（去掉整段全持久那一个，它就是下一段的空子集）；
//! 甲二快档（原地写任意组合 × 单元写全不落或全落）原地写 k 个、其中 m 个取 3 态、单元写 c 个：3^m · 2^(k−m) ·（c > 0 时 2，否则 1）− 1。
//! 整条流再加全部持久那一个。m 全为 0 时两式就是今天的闭式（第一条用例拿 `crash::closed_form_state_count` 与
//! `crash::layer0_state_count` 对拍）。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use singlefs_checker_tier::crash::{
    full_expansion, layer0_state_count, quick_tier_expansion, Layer0SegmentExpansion,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::recovery::PoolReader;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, MemoryPool, RetainedWrite, WrittenContents,
    SECTOR_BYTES,
};
use singlefs_harness::segments::StepKind;

/// 一次写撕裂时会不会出「新旧都读不出」的第三态（判据见模块文档的三条）。
fn is_tearable_in_place_overwrite(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    write_index: usize,
) -> bool {
    let write = &writes[write_index];
    let is_copy_on_write = match write.kind {
        StepKind::UnitWrite => true,
        StepKind::ZeroFill
        | StepKind::JournalRecord
        | StepKind::RootRecordFua
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => false,
    };
    if is_copy_on_write || write.length_in_bytes() <= SECTOR_BYTES {
        return false;
    }
    let write_end = write.offset.0 + write.length_in_bytes();
    let base_holds_nonzero_bytes_there = PoolReader::read(
        base,
        write.device,
        write.offset,
        usize::try_from(write.length_in_bytes()).expect("写长装得进 usize"),
    )
    .is_some_and(|bytes| bytes.iter().any(|byte| *byte != 0));
    let an_earlier_write_with_bytes_overlaps = writes[..write_index].iter().any(|earlier| {
        let earlier_has_bytes = match earlier.contents {
            WrittenContents::Bytes(_) => true,
            WrittenContents::Zeros { .. } => false,
        };
        earlier_has_bytes
            && earlier.device == write.device
            && earlier.offset.0 < write_end
            && write.offset.0 < earlier.offset.0 + earlier.length_in_bytes()
    });
    base_holds_nonzero_bytes_there || an_earlier_write_with_bytes_overlaps
}

/// 一段按 `expansion` 展开、原地覆写的写取 3 态时的状态数（式子见模块文档）。
fn segment_state_count_with_the_torn_third_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segment: &[usize],
    expansion: Layer0SegmentExpansion,
) -> u128 {
    let states_of_the_writes = |write_indexes: &[usize]| -> u128 {
        write_indexes
            .iter()
            .map(|write_index| {
                if is_tearable_in_place_overwrite(base, writes, *write_index) {
                    3u128
                } else {
                    2u128
                }
            })
            .product()
    };
    match expansion {
        Layer0SegmentExpansion::NotExpanded => 0,
        Layer0SegmentExpansion::EveryProperSubset => states_of_the_writes(segment) - 1,
        Layer0SegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
            let (copy_on_write_writes, in_place_writes): (Vec<usize>, Vec<usize>) = segment
                .iter()
                .copied()
                .partition(|write_index| match writes[*write_index].kind {
                    StepKind::UnitWrite => true,
                    StepKind::ZeroFill
                    | StepKind::JournalRecord
                    | StepKind::RootRecordFua
                    | StepKind::SystemConfigurationSlot
                    | StepKind::Barrier => false,
                });
            let copy_on_write_choices: u128 = if copy_on_write_writes.is_empty() {
                1
            } else {
                2
            };
            states_of_the_writes(&in_place_writes) * copy_on_write_choices - 1
        }
    }
}

/// 整条流：各段按 `expansion` 展开的状态数之和，再加全部持久那一个。
fn layer0_state_count_with_the_torn_third_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
) -> u128 {
    1 + segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            segment_state_count_with_the_torn_third_state(
                base,
                writes,
                segment,
                expansion(segment_index, segment),
            )
        })
        .sum::<u128>()
}

/// 手摆一次普通写（`length_in_bytes` 个 `fill` 字节）。
fn plain_write(
    device: u32,
    kind: StepKind,
    offset: u64,
    length_in_bytes: usize,
    fill: u8,
) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(device),
        kind,
        is_force_unit_access: kind == StepKind::RootRecordFua,
        offset: DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(vec![fill; length_in_bytes]),
    }
}

const SYSTEM_CONFIGURATION_SLOT_OFFSET: u64 = 4096;
const ROOT_SLOT_OFFSET: u64 = 1 << 20;
const JOURNAL_SLOT_OFFSET: u64 = 16 << 20;
const UNIT_SLOT_OFFSET: u64 = 784 << 20;

/// 一条没有一次写罩住旧内容的流：每个写都取 2 态，两式都退回今天的闭式（与 harness 自己算的逐个相等）。
#[test]
fn without_any_tearable_overwrite_the_counts_are_todays_closed_forms() {
    let base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 1 << 30);
    let writes = vec![
        plain_write(0, StepKind::UnitWrite, UNIT_SLOT_OFFSET, 32768, 1),
        plain_write(1, StepKind::UnitWrite, UNIT_SLOT_OFFSET, 32768, 1),
        plain_write(0, StepKind::JournalRecord, JOURNAL_SLOT_OFFSET, 4096, 2),
        plain_write(1, StepKind::JournalRecord, JOURNAL_SLOT_OFFSET, 4096, 2),
        plain_write(0, StepKind::RootRecordFua, ROOT_SLOT_OFFSET, 512, 3),
        plain_write(
            0,
            StepKind::SystemConfigurationSlot,
            SYSTEM_CONFIGURATION_SLOT_OFFSET,
            4096,
            4,
        ),
        plain_write(
            1,
            StepKind::SystemConfigurationSlot,
            SYSTEM_CONFIGURATION_SLOT_OFFSET,
            4096,
            4,
        ),
    ];
    let segments = vec![vec![0, 1, 2, 3], vec![4], vec![5, 6]];
    for write_index in 0..writes.len() {
        assert!(
            !is_tearable_in_place_overwrite(&base, &writes, write_index),
            "写表下标 {write_index}：基镜像全 0、没有更早的写与它重叠，不是原地覆写"
        );
    }
    assert_eq!(
        layer0_state_count_with_the_torn_third_state(&base, &writes, &segments, &full_expansion),
        u128::from(closed_form_state_count(&segments))
    );
    assert_eq!(
        layer0_state_count_with_the_torn_third_state(
            &base,
            &writes,
            &segments,
            &quick_tier_expansion
        ),
        u128::from(layer0_state_count(
            &writes,
            &segments,
            &quick_tier_expansion
        ))
    );
}

/// 原地覆写的写取 3 态、别的写取 2 态：基镜像上系统配置槽与根槽有旧内容，第三段再覆写第一段写过的那个 journal 槽。
/// 系统配置槽写（4096，罩住旧内容）与第二次 journal 写（罩住第一次）取 3 态；第一次 journal 写（那里原来是 0）、根槽写
/// （一个扇区，撕不开）、单元写（COW）取 2 态。
#[test]
fn a_tearable_in_place_overwrite_takes_a_third_state_and_every_other_write_two() {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 1 << 30);
    let old_device_zero = base.devices.get_mut(&DeviceIdentity(0)).expect("盘 0");
    old_device_zero.write(
        DeviceOffsetInBytes(SYSTEM_CONFIGURATION_SLOT_OFFSET),
        &[0x5Au8; 4096],
    );
    old_device_zero.write(DeviceOffsetInBytes(ROOT_SLOT_OFFSET), &[0x6Bu8; 512]);
    let writes = vec![
        plain_write(
            0,
            StepKind::SystemConfigurationSlot,
            SYSTEM_CONFIGURATION_SLOT_OFFSET,
            4096,
            1,
        ),
        plain_write(0, StepKind::JournalRecord, JOURNAL_SLOT_OFFSET, 4096, 2),
        plain_write(0, StepKind::UnitWrite, UNIT_SLOT_OFFSET, 32768, 3),
        plain_write(1, StepKind::UnitWrite, UNIT_SLOT_OFFSET, 32768, 3),
        plain_write(0, StepKind::RootRecordFua, ROOT_SLOT_OFFSET, 512, 4),
        plain_write(0, StepKind::JournalRecord, JOURNAL_SLOT_OFFSET, 4096, 5),
    ];
    let segments = vec![vec![0, 1, 2, 3], vec![4], vec![5]];
    let tearable: Vec<bool> = (0..writes.len())
        .map(|write_index| is_tearable_in_place_overwrite(&base, &writes, write_index))
        .collect();
    assert_eq!(
        tearable,
        vec![true, false, false, false, false, true],
        "系统配置槽写与第二次 journal 写是原地覆写；第一次 journal 写、两个单元写、一个扇区的根槽写不是"
    );
    // 全量：第一段 3 · 2 · 2 · 2 − 1 = 23，第二段 2 − 1 = 1，第三段 3 − 1 = 2，再加全部持久那一个。
    assert_eq!(
        layer0_state_count_with_the_torn_third_state(&base, &writes, &segments, &full_expansion),
        1 + 23 + 1 + 2
    );
    // 甲二快档：第一段原地写（系统配置槽 3 态、journal 2 态）6 × 单元写全不落或全落 2 − 1 = 11，其余两段同全量。
    assert_eq!(
        layer0_state_count_with_the_torn_third_state(
            &base,
            &writes,
            &segments,
            &quick_tier_expansion
        ),
        1 + 11 + 1 + 2
    );
    // 今天的数（每个写 2 态）：全量 1 + 15 + 1 + 1，甲二 1 + 7 + 1 + 1。
    assert_eq!(closed_form_state_count(&segments), 1 + 15 + 1 + 1);
    assert_eq!(
        layer0_state_count(&writes, &segments, &quick_tier_expansion),
        1 + 7 + 1 + 1
    );
}

/// 量：新池新建文件那条流（层 0 从 mkfs 之后枚举的整条流，段序列 `2+2+1+2+2+1+2+24+2+1+2`；C577 之前暖机第二次的轮换与新池新建文件的
/// 单元写同段，`2+2+1+2+2+1+26+2+1+2`）补第三态之后的状态数。
/// 这条流上取 3 态的只有系统配置槽写（8 次，每次都罩住 mkfs 写下的旧槽）；journal 记录都写在 mkfs 清过的环里、没有覆写，
/// 根槽写一个扇区、单元写 COW，都照旧 2 态。
#[test]
fn the_new_pool_file_creation_stream_counts_with_the_torn_third_state_as_measured() {
    let pool = common::build_pool("torn-third-state-count");
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &common::geometry(),
    );
    let mut tearable_writes_by_kind: Vec<(&str, usize, usize)> = Vec::new();
    for kind in [
        StepKind::SystemConfigurationSlot,
        StepKind::JournalRecord,
        StepKind::RootRecordFua,
        StepKind::UnitWrite,
    ] {
        let of_this_kind: Vec<usize> = (0..writes.len())
            .filter(|write_index| writes[*write_index].kind == kind)
            .collect();
        let tearable = of_this_kind
            .iter()
            .filter(|write_index| is_tearable_in_place_overwrite(&base, &writes, **write_index))
            .count();
        tearable_writes_by_kind.push((kind.name(), of_this_kind.len(), tearable));
    }
    let today_full = u128::from(closed_form_state_count(&segments));
    let today_quick_tier = u128::from(layer0_state_count(
        &writes,
        &segments,
        &quick_tier_expansion,
    ));
    let torn_full =
        layer0_state_count_with_the_torn_third_state(&base, &writes, &segments, &full_expansion);
    let torn_quick_tier = layer0_state_count_with_the_torn_third_state(
        &base,
        &writes,
        &segments,
        &quick_tier_expansion,
    );
    println!(
        "MEASURE stream=new_pool_file_creation writes={} segments={} tearable_by_kind={tearable_writes_by_kind:?} \
         today_full={today_full} torn_full={torn_full} today_quick_tier={today_quick_tier} torn_quick_tier={torn_quick_tier}",
        writes.len(),
        segments.len()
    );
    assert_eq!(
        tearable_writes_by_kind,
        vec![
            ("system_configuration_slot", 8, 8),
            ("journal_record", 6, 0),
            ("root_record_fua", 3, 0),
            ("unit_write", 24, 0),
        ]
    );
    assert_eq!(
        today_full, 16_777_240,
        "今天的全量闭式：1 + (3 + 3 + 1 + 3 + 3 + 1 + 3 + 3 + 1 + 3) + (2^24 − 1)（C577 之前 26 写段 2^26 − 1，67108885，\
         layout/01-first-txn.md 八那一行要跟着改）"
    );
    assert_eq!(
        (torn_full, today_quick_tier, torn_quick_tier),
        (16_777_260, 26, 46),
        "补第三态：全量四个系统配置槽段各 3² − 1、24 写的单元段 2²⁴ − 1；甲二单元段全不落 1、系统配置槽段各 3² − 1 \
         （C577 之前轮换与单元同段：全量那一段 3² · 2²⁴ − 1、甲二 3² · 2 − 1，共 150994980、29、54）"
    );
}
