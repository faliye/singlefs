//! harness 测试文件 `crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor, ShadowLedger,
};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 复用回收槽的那段历史（攻方 `research/prompts/k3-9-barrier-r1-opus-model/` 的 floorreuse4 场景）：两块单元区 384 槽的小盘，
/// mkfs、挂载、第一个文件、覆盖写 4 次、抬 F 到上限（现行 txg − 3），抬 F 之后换下的槽可回收；再覆盖写一次（被判的那一次），
/// 它的单元落在盘上已有旧单元的槽里。交回被判那一次之前的池（之前的写全持久）、被判那一次的写表与切段。
pub struct FloorRaisedThenAPublishReusingReclaimedSlots {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
}

/// 第 `round` 次发布的内容：长度不同、字节不同，每次覆盖写都换下上一版的数据单元。
pub fn content_of_round(round: usize, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 13 + round * 7 + 3) % 251).expect("小于 251"))
        .collect()
}

/// 每次发布的写入时间：照 E142 那个固定时刻起，每次往后一秒。
pub const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

pub fn floor_raised_then_a_publish_reusing_reclaimed_slots(
) -> FloorRaisedThenAPublishReusingReclaimedSlots {
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let parameters = width.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(
                            width.device_bytes(),
                            PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
                        ),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("mkfs 之后的可写挂载");
    let PoolVersion::WithoutFile(version_without_file) = &mounted.current else {
        panic!("mkfs 之后挂上的那一版还没有文件");
    };
    let root_without_file = version_without_file.root;
    let record_bytes_without_file = version_without_file.record_bytes.clone();
    let instance = mounted.output.instance;
    let first_content = content_of_round(1, 3000);
    let mut current = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            &root_without_file,
            FirstFile {
                content: &first_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS,
            },
            instance,
            &record_bytes_without_file,
        )
        .expect("第一个文件")
    };
    for round in 0..4u64 {
        let round_index = usize::try_from(round).expect("小于 4");
        let content = content_of_round(round_index + 10, 3000 + round_index);
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1 + round,
            },
            instance,
        )
        .expect("覆盖写");
    }
    // 抬 F 的上限是现行 txg − 3（攻方先试时 `RollbackFloorAboveCeiling` 报的 ceiling）。
    let floor_at_the_ceiling = CheckpointTxg(current.root.checkpoint_txg.0 - 3);
    raise_rollback_floor(
        &parameters,
        &mut devices,
        &mut mounted.allocator,
        &mut current,
        floor_at_the_ceiling,
        ShadowLedger::On,
    )
    .expect("抬 F 到上限");
    let operations_before_the_judged_publish = stream.operation_count();
    let judged_content = content_of_round(2, 4000);
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &judged_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 5,
            },
            instance,
        )
        .expect("被判的那一次覆盖写");
    }
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        width.device_bytes(),
    );
    base.apply(&operations[..operations_before_the_judged_publish]);
    let (writes, segments) = writes_and_segments(
        &operations[operations_before_the_judged_publish..],
        &width.fixed_geometry(),
    );
    FloorRaisedThenAPublishReusingReclaimedSlots {
        base,
        writes,
        segments,
    }
}

/// C592（单独拿掉单元写与记录之间那道屏障，今天没有判据会红）的那一格：复用回收槽的历史上，被判那一次发布的每一段各扣下一个写
/// （更早的段整段持久、这一段别的写持久、更晚的段一个都没持久），池级 checker 一条不红。这道屏障拿掉之后单元写与记录并成一段，
/// 「复用槽上那个数据单元的一份没落、记录落了」那个状态进了枚举域，下一次挂载会先施加的那一版的分配记录与它罩住的单元不同源，
/// I-3.10 在结局正确的状态上红（攻方 floorreuse4：删屏障 271 个状态红 48，带屏障 73 个状态 0；`crates/mutations.tsv` 删屏障那一行点名这一条）。
// crash-case-check:not-a-crash-case 只摆被判那一次发布的单扣状态，几十个，debug 下单跑几秒
pub fn with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations(
    images: &mut PoolCheckerImages,
) {
    let judged = floor_raised_then_a_publish_reusing_reclaimed_slots();
    let reused_unit_writes = judged
        .writes
        .iter()
        .filter(|write| {
            write.kind == StepKind::UnitWrite
                && !judged.base.devices[&write.device]
                    .written_sectors_in(write.offset, write.length_in_bytes())
                    .is_empty()
        })
        .count();
    assert!(
        reused_unit_writes >= 1,
        "前提：被判那一次至少有一个单元写落在盘上已写过的槽里（复用回收的槽）"
    );
    let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
    // 迭代上界是被判那一次发布的写数；每一轮从同一份基线起摆一个状态，不跨轮带状态。
    for (segment_index, segment) in judged.segments.iter().enumerate() {
        for withheld in segment {
            let mut persisted = vec![false; judged.writes.len()];
            for earlier_segment in &judged.segments[..segment_index] {
                for write in earlier_segment {
                    persisted[*write] = true;
                }
            }
            for write in segment {
                persisted[*write] = write != withheld;
            }
            persisted_sets.push(persisted);
        }
    }
    assert_eq!(
        persisted_sets.len(),
        judged.writes.len(),
        "每个写各扣下一次，状态数等于写数"
    );
    // 带着单元写与记录之间那道屏障，单扣状态上池级 checker 一条不红
    images.hand_out_crash_states(
        "被判那一次发布每一段各扣下一个写",
        || {
            (
                judged.base.clone(),
                judged.writes.clone(),
                judged.segments.clone(),
                persisted_sets.clone(),
            )
        },
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
