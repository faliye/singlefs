//! C331（择根倒挂压过已确认的写）在今天 crates/ 上的结局（调查员复查，2026-09-28，草稿：只在仓副本里）。
//!
//! 历史：实例 1 暖机（txg 1、2）、A（txg 3，jsn 3）→ 重开，实例 2 写行（txg 4）、暖机（txg 5）、B（txg 6）、C（txg 7），全部确认返回；
//! 之后按各场景让实例 2 的根 / 记录 / 系统配置槽在下一次挂载时读不出，新实例（实例 3）能可写挂载就写 E、确认返回，崩溃（关镜像），
//! 撤掉读故障再重开，看 E 在不在。
mod common;

use common::{
    build_pool, parameters, publish_overwrite_in_process, unreadable_root_slot_of,
    with_unreadable_ranges, without_unreadable_ranges, BuiltPool, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_writable, MountError, Mounted, NewerPublishWitness, ReadStageSettled, RollbackTarget,
    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::PoolReader;
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

const B_BYTES: usize = 4100;
const C_BYTES: usize = 2500;
const D_BYTES: usize = 3300;
const E_BYTES: usize = 3700;
fn b_content() -> Vec<u8> {
    content_of(B_BYTES, 3)
}
fn c_content() -> Vec<u8> {
    content_of(C_BYTES, 11)
}
fn d_content() -> Vec<u8> {
    content_of(D_BYTES, 17)
}
fn e_content() -> Vec<u8> {
    content_of(E_BYTES, 29)
}

struct History {
    pool: BuiltPool,
    a: TransactionOutput,
    b: TransactionOutput,
    c: TransactionOutput,
    /// 实例 2 的写行与暖机那两次发布的 txg。
    second_instance_mount_txgs: Vec<CheckpointTxg>,
}

fn history(tag: &str) -> History {
    let mut pool = build_pool(tag);
    let a = pool.output.clone();
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("实例 2 可写挂载");
    pool.devices = Some(devices);
    let second_instance_mount_txgs =
        std::iter::once(mounted.output.row_publish.root().checkpoint_txg)
            .chain(
                mounted
                    .output
                    .warm_up_publishes
                    .iter()
                    .map(|v| v.root().checkpoint_txg),
            )
            .collect::<Vec<_>>();
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    let previous = pool.output.clone();
    let b = publish_overwrite_in_process(
        &mut pool,
        &previous,
        &b_content(),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(2),
    )
    .expect("B");
    let c = publish_overwrite_in_process(
        &mut pool,
        &b,
        &c_content(),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(2),
    )
    .expect("C");
    pool.output = c.clone();
    eprintln!(
        "history: A=(1,{},jsn {}) second-instance mount txgs={:?} B=(2,{},jsn {}) C=(2,{},jsn {})",
        a.root.checkpoint_txg.0,
        a.record.counter,
        second_instance_mount_txgs,
        b.root.checkpoint_txg.0,
        b.record.counter,
        c.root.checkpoint_txg.0,
        c.record.counter
    );
    History {
        pool,
        a,
        b,
        c,
        second_instance_mount_txgs,
    }
}

fn root_slots(
    txgs: std::ops::RangeInclusive<u64>,
    failing: FailingReadsOfARange,
) -> Vec<UnreadableRange> {
    txgs.map(|txg| unreadable_root_slot_of(CheckpointTxg(txg), failing))
        .collect()
}

fn records(
    counters: std::ops::RangeInclusive<u64>,
    failing: FailingReadsOfARange,
) -> Vec<UnreadableRange> {
    counters
        .flat_map(|counter| {
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(move |device| UnreadableRange {
                    device,
                    offset_in_bytes: record_offset(
                        counter,
                        parameters().geometry.journal_ring_bytes,
                    )
                    .0,
                    length_in_bytes: JOURNAL_RECORD_BYTES,
                    failing_reads: failing,
                })
        })
        .collect()
}

/// 每块盘两槽系统配置（`newest_only` 时只取世代号最大那一槽，即见证最近一次发布的那一槽）。
fn system_configuration_slots<Reader: PoolReader>(
    reader: &Reader,
    newest_only: bool,
    devices: &[DeviceIdentity],
    failing: FailingReadsOfARange,
) -> Vec<UnreadableRange> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    devices
        .iter()
        .flat_map(|device| {
            let slots = singlefs_core::recovery::verified_system_configuration_slots(
                reader,
                *device,
                spacing,
                &parameters().filesystem_identifier,
            );
            let newest = slots
                .iter()
                .map(|s| s.quantities.slot_generation)
                .max()
                .expect("有自证过的槽");
            let offsets: Vec<u64> = if newest_only {
                vec![(newest % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing]
            } else {
                vec![0, spacing]
            };
            for s in &slots {
                eprintln!(
                    "  config slot device {} gen {} tail {} instance {}",
                    device.0,
                    s.quantities.slot_generation,
                    s.quantities.journal_tail,
                    s.quantities.journal_instance.0
                );
            }
            offsets.into_iter().map(move |offset| UnreadableRange {
                device: *device,
                offset_in_bytes: offset,
                length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
                failing_reads: failing,
            })
        })
        .collect()
}

/// 读回现行那一版的文件：交回 (实例, txg, 字节)。
fn read_only_view<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    length: usize,
) -> (u32, u64, Vec<u8>) {
    let read_only = mount_read_only(devices).expect("只读挂载");
    let file = read_only
        .mounted
        .open_file(devices, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    let bytes = file
        .read_at(
            devices,
            FileOffsetInBytes(0),
            u64::try_from(length).expect("长度"),
        )
        .map(|out| out.bytes)
        .unwrap_or_default();
    (
        read_only.effective_root.instance.0,
        read_only.effective_root.checkpoint_txg.0,
        bytes,
    )
}

fn describe(mounted: &Result<Mounted, MountError>) -> String {
    match mounted {
        Ok(m) => format!(
            "Ok: instance {} chosen ({},{}) effective ({},{}) row txg {} warm-ups {:?} rereads {:?} isolated {:?} abandoned_unreadable {}",
            m.output.instance.0,
            m.output.chosen_root.instance.0, m.output.chosen_root.checkpoint_txg.0,
            m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0,
            m.output.row_publish.root().checkpoint_txg.0,
            m.output.warm_up_publishes.iter().map(|v| v.root().checkpoint_txg.0).collect::<Vec<_>>(),
            m.output.rereads, m.output.isolated_slots_per_device, m.output.abandoned_roots_unreadable
        ),
        Err(e) => format!("Err: {e:?}"),
    }
}

/// 新实例在挂载交回的分配器上覆盖写一次（确认返回）。
fn publish_e<Device: BlockDevice>(
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mounted: &mut Mounted,
) -> TransactionOutput {
    let params = parameters();
    let previous = mounted.current.file_version().expect("带文件").clone();
    let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
    let e = publish_overwrite(
        &mut writer,
        &mut mounted.allocator,
        &previous,
        FirstFile {
            content: &e_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
        },
        mounted.output.instance,
    )
    .expect("新实例覆盖写 E 确认返回");
    eprintln!(
        "E = ({},{}) jsn {}",
        e.root.instance.0, e.root.checkpoint_txg.0, e.record.counter
    );
    e
}

/// 新实例那次可写挂载的结局。
#[derive(Debug)]
enum NewInstanceMount {
    Refused(String),
    Mounted {
        chosen: (u32, u64),
        effective: (u32, u64),
        row_txg: u64,
        read_stage: ReadStageSettled,
    },
}

/// 新实例挂载之后到最后一次重开的结局。
struct Outcome {
    mount: NewInstanceMount,
    /// 新实例确认返回的 E。
    e: Option<TransactionOutput>,
    /// 撤故障、崩溃之后冷重开：只读择到的 (实例, txg)、读回的是不是 E。
    final_read_only: Option<(u32, u64, bool)>,
    /// 同一次冷重开的可写挂载：择到的 (实例, txg) 或错。
    final_writable: Result<(u32, u64), String>,
    /// 最后一次重开时根环里读得出的根。
    final_roots: Vec<(u32, u64)>,
    pool: BuiltPool,
    unreadable: SharedUnreadableRanges,
}

fn readable_roots_of<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
) -> Vec<(u32, u64)> {
    let params = parameters();
    let mut roots: Vec<(u32, u64)> = singlefs_core::recovery::readable_roots(
        devices,
        &params.region_devices,
        &params.geometry,
        &params.filesystem_identifier,
    )
    .iter()
    .map(|r| (r.instance.0, r.checkpoint_txg.0))
    .collect();
    roots.sort_by_key(|(instance, txg)| (*txg, *instance));
    roots
}

/// 可写挂载时点名的读故障在位；挂载一交回就撤（「那些根以后又读得出」）；做成了就写 E、确认返回，崩溃（关镜像），冷重开。
/// 挂载被拒时撤故障冷重开一次，看择到哪一版。
fn run_c331_history(mut pool: BuiltPool, ranges: Vec<UnreadableRange>) -> Outcome {
    let unreadable = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let mounted = mount_writable(&parameters(), &mut devices);
    eprintln!("new-instance mount: {}", describe(&mounted));
    let (mount, e) = match mounted {
        Err(error) => {
            if mount_read_only(&devices).is_ok() {
                let under_faults = read_only_view(&devices, common::FILE_BYTES);
                eprintln!(
                    "read-only under the same faults: ({},{}) reads back A = {}",
                    under_faults.0,
                    under_faults.1,
                    under_faults.2 == common::file_content()
                );
            }
            unreadable.lift();
            pool.devices = Some(without_unreadable_ranges(devices));
            (NewInstanceMount::Refused(format!("{error:?}")), None)
        }
        Ok(mut mounted) => {
            let mount = NewInstanceMount::Mounted {
                chosen: (
                    mounted.output.chosen_root.instance.0,
                    mounted.output.chosen_root.checkpoint_txg.0,
                ),
                effective: (
                    mounted.output.effective_root.instance.0,
                    mounted.output.effective_root.checkpoint_txg.0,
                ),
                row_txg: mounted.output.row_publish.root().checkpoint_txg.0,
                read_stage: mounted.output.rereads.read_stage,
            };
            unreadable.lift();
            let e = publish_e(&mut devices, &mut mounted);
            pool.devices = Some(without_unreadable_ranges(devices));
            (mount, Some(e))
        }
    };
    let mut devices = pool.reopen_recorded();
    let final_roots = readable_roots_of(&devices);
    let final_tails: Vec<(u32, u64, u64)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .iter()
        .flat_map(|device| {
            singlefs_core::recovery::verified_system_configuration_slots(
                &devices,
                *device,
                u64::from(parameters().geometry.fixed_structure_slot_spacing),
                &parameters().filesystem_identifier,
            )
            .into_iter()
            .map(move |slot| {
                (
                    device.0,
                    slot.quantities.slot_generation,
                    slot.quantities.journal_tail,
                )
            })
        })
        .collect();
    eprintln!("final system configuration (device, generation, tail): {final_tails:?}");
    let final_read_only = mount_read_only(&devices).ok().map(|read_only| {
        let bytes = read_only
            .mounted
            .open_file(&devices, InodeNumber(FIRST_INODE_NUMBER))
            .ok()
            .and_then(|file| {
                file.read_at(
                    &devices,
                    FileOffsetInBytes(0),
                    u64::try_from(E_BYTES).expect("长度"),
                )
                .ok()
            })
            .map(|out| out.bytes)
            .unwrap_or_default();
        (
            read_only.effective_root.instance.0,
            read_only.effective_root.checkpoint_txg.0,
            bytes == e_content(),
        )
    });
    let writable = mount_writable(&parameters(), &mut devices);
    eprintln!(
        "final roots {final_roots:?}; final read-only {final_read_only:?}; final writable: {}",
        describe(&writable)
    );
    let final_writable = writable
        .map(|m| {
            (
                m.output.effective_root.instance.0,
                m.output.effective_root.checkpoint_txg.0,
            )
        })
        .map_err(|error| format!("{error:?}"));
    pool.devices = Some(devices);
    Outcome {
        mount,
        e,
        final_read_only,
        final_writable,
        final_roots,
        pool,
        unreadable,
    }
}

/// 实例 2 全部根槽（写行 txg 4 起到 C txg 7）与全部记录（jsn 4–7，两份）。
fn every_root_and_record_of_the_second_instance(
    h: &History,
    failing: FailingReadsOfARange,
) -> Vec<UnreadableRange> {
    let mut ranges = root_slots(
        h.second_instance_mount_txgs[0].0..=h.c.root.checkpoint_txg.0,
        failing,
    );
    ranges.extend(records(
        h.a.record.counter + 1..=h.c.record.counter,
        failing,
    ));
    ranges
}

fn assert_history_shape(h: &History) {
    assert_eq!(
        (
            h.a.root.checkpoint_txg.0,
            h.a.record.counter,
            h.second_instance_mount_txgs.clone(),
            h.b.root.checkpoint_txg.0,
            h.c.root.checkpoint_txg.0,
            h.c.record.counter
        ),
        (3, 3, vec![CheckpointTxg(4), CheckpointTxg(5)], 6, 7, 7),
        "A txg 3 jsn 3；实例 2 写行 4、暖机 5；B 6；C 7 jsn 7"
    );
}

fn witness_refusal(selected: (u32, u64), selected_last_record: u64, witnessed: u64) -> String {
    let reading = SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: InstanceGeneration(selected.0),
            checkpoint_txg: CheckpointTxg(selected.1),
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: witnessed,
            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter: selected_last_record,
            },
        },
    };
    format!("{:?}", MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: reading, reread: reading },
    )))
}

/// C331 那段历史（乙-配置续 罩住的那一形）：实例 2 的全部根与记录持续读不出 ⇒ 见证 7 > A 的末条 3，重读一次仍判真，拒可写；
/// 撤故障之后可写挂载择 C。新实例没写成任何东西，没有已确认的写可被压过。
#[test]
fn c331_history_refuses_the_writable_mount_while_every_root_and_record_of_the_newer_instance_stays_unreadable(
) {
    let h = history("c331-refused");
    assert_history_shape(&h);
    let ranges = every_root_and_record_of_the_second_instance(&h, FailingReadsOfARange::Every);
    let outcome = run_c331_history(h.pool, ranges);
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Refused(text) if *text == witness_refusal((1, 3), 3, 7)),
        "{:?}",
        outcome.mount
    );
    assert_eq!(outcome.final_writable, Ok((2, 7)), "撤故障之后择 C");
}

/// 只有实例 2 的根读不出、记录读得出：重放不跨实例（实例 2 第一条记录的反向链从 0 起），所选那一版仍是 A ⇒ 同样拒可写。
#[test]
fn c331_history_with_only_the_roots_unreadable_also_refuses_because_the_replay_stays_on_the_chosen_instance(
) {
    let h = history("c331-roots-only");
    assert_history_shape(&h);
    let ranges = root_slots(
        h.second_instance_mount_txgs[0].0..=h.c.root.checkpoint_txg.0,
        FailingReadsOfARange::Every,
    );
    let outcome = run_c331_history(h.pool, ranges);
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Refused(text) if *text == witness_refusal((1, 3), 3, 7)),
        "{:?}",
        outcome.mount
    );
    assert_eq!(outcome.final_writable, Ok((2, 7)));
}

/// 每段只坏第一次读（产品路径立即重读）：重读择 C，实例 3 写 E（txg 11）确认返回；撤故障重开择 E。
#[test]
fn c331_history_with_a_one_read_fault_rereads_into_the_newest_root_and_the_new_instance_write_survives(
) {
    let h = history("c331-one-read-fault");
    assert_history_shape(&h);
    let ranges =
        every_root_and_record_of_the_second_instance(&h, FailingReadsOfARange::OnlyTheFirst);
    let outcome = run_c331_history(h.pool, ranges);
    assert!(
        matches!(
            &outcome.mount,
            NewInstanceMount::Mounted {
                chosen: (2, 7),
                effective: (2, 7),
                row_txg: 8,
                read_stage: ReadStageSettled::OnTheOneReread { .. }
            }
        ),
        "{:?}",
        outcome.mount
    );
    let e = outcome.e.expect("E 确认返回");
    assert_eq!((e.root.instance.0, e.root.checkpoint_txg.0), (3, 11));
    assert_eq!(outcome.final_read_only, Some((3, 11, true)));
    assert_eq!(outcome.final_writable, Ok((3, 11)));
}

/// 除重读那一次之外每次都坏：重读择 C；之后直接读盘的几处又看不见实例 2 的根，首个 txg 由重读那一遍的记录定为 8；E 活下来。
#[test]
fn roots_and_records_readable_only_on_the_reread_are_chosen_and_the_new_instance_write_survives() {
    let h = history("c331-readable-only-on-the-reread");
    assert_history_shape(&h);
    let ranges =
        every_root_and_record_of_the_second_instance(&h, FailingReadsOfARange::AllButTheNth(2));
    let outcome = run_c331_history(h.pool, ranges);
    assert!(
        matches!(
            &outcome.mount,
            NewInstanceMount::Mounted {
                chosen: (2, 7),
                row_txg: 8,
                read_stage: ReadStageSettled::OnTheOneReread { .. },
                ..
            }
        ),
        "{:?}",
        outcome.mount
    );
    assert_eq!(outcome.final_read_only, Some((3, 11, true)));
    assert_eq!(outcome.final_writable, Ok((3, 11)));
}

/// 盘 1 两槽系统配置持续读不出、实例 2 的根与记录持续读不出：盘 0 仍见证 7 ⇒ 拒可写。
#[test]
fn one_device_with_both_system_configuration_slots_unreadable_still_witnesses_the_newer_instance_and_refuses(
) {
    let mut h = history("c331-one-device-config");
    assert_history_shape(&h);
    let devices = h.pool.reopen_recorded();
    let mut ranges = system_configuration_slots(
        &devices,
        false,
        &[DeviceIdentity(1)],
        FailingReadsOfARange::Every,
    );
    h.pool.devices = Some(devices);
    ranges.extend(every_root_and_record_of_the_second_instance(
        &h,
        FailingReadsOfARange::Every,
    ));
    let outcome = run_c331_history(h.pool, ranges);
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Refused(text) if *text == witness_refusal((1, 3), 3, 7)),
        "{:?}",
        outcome.mount
    );
}

/// 写系统配置槽（每块盘前两个槽距）在 `armed` 时报块设备错：造「根 FUA 之后、系统配置轮换之前」那一格（发布没确认返回）。
struct FailingSystemConfigurationWrites<Inner: BlockDevice> {
    inner: Inner,
    armed: bool,
}

impl<Inner: BlockDevice> BlockDevice for FailingSystemConfigurationWrites<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
        if self.armed && offset.0 < 2 * spacing {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "系统配置槽写失败（用例造的）",
            )));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 系统配置没见证过的最新根：实例 2 在 C 之后发 D（txg 8），D 的根 FUA 之后轮换写系统配置失败（D 没确认返回），崩溃。
/// 重开时 D 的根与记录读不出 ⇒ 见证 7 = C 的末条，判假、不重读，从 C 可写挂载；实例 3 写行 txg 8 落在 D 的根槽上把 D 盖掉；E（txg 11）活下来。
#[test]
fn a_newest_root_left_unwitnessed_by_a_failed_rotation_is_overwritten_by_the_new_instance_row_publish(
) {
    let mut h = history("c331-unwitnessed");
    assert_history_shape(&h);
    let mut wrapped: Vec<_> = h
        .pool
        .devices
        .take()
        .expect("开着")
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FailingSystemConfigurationWrites { inner, armed: true },
            )
        })
        .collect();
    let params = parameters();
    let d = {
        let mut writer = PoolWriter::new(&params, wrapped.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut h.pool.allocator,
            &h.c,
            FirstFile {
                content: &d_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
            },
            InstanceGeneration(2),
        )
    };
    assert!(d.is_err(), "D 在轮换那一步失败、没确认返回");
    h.pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    let devices = h.pool.reopen_recorded();
    assert!(
        readable_roots_of(&devices).contains(&(2, 8)),
        "D 的根落了盘"
    );
    h.pool.devices = Some(devices);
    let mut ranges = root_slots(8..=8, FailingReadsOfARange::Every);
    ranges.extend(records(8..=8, FailingReadsOfARange::Every));
    let outcome = run_c331_history(h.pool, ranges);
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Mounted { chosen: (2, 7), effective: (2, 7), row_txg: 8, read_stage: ReadStageSettled::OnTheFirstRead { first_read } } if !first_read.witnesses_a_publish_newer_than_the_selected_version()),
        "{:?}",
        outcome.mount
    );
    assert!(
        outcome.final_roots.contains(&(3, 8)) && !outcome.final_roots.contains(&(2, 8)),
        "D 的根槽被实例 3 的写行盖掉：{:?}",
        outcome.final_roots
    );
    assert_eq!(outcome.final_read_only, Some((3, 11, true)));
    assert_eq!(outcome.final_writable, Ok((3, 11)));
}

/// 两块盘见证 C 的那一槽（世代号最大）持续读不出、C 的根与记录读不出：见证退到 6 = B 的末条，判假，从 B 可写挂载——
/// 实例 2 已确认的 C 丢了、挂载不拒；实例 3 写行 txg 7 盖掉 C 的根槽，E（txg 9）活下来。压过的是较旧实例的写，不是新实例的。
#[test]
fn the_newest_system_configuration_slot_unreadable_on_both_devices_loses_the_older_instance_acknowledged_publish_but_not_the_new_instance_write(
) {
    let mut h = history("c331-newest-config-slots");
    assert_history_shape(&h);
    let devices = h.pool.reopen_recorded();
    let mut ranges = system_configuration_slots(
        &devices,
        true,
        &[DeviceIdentity(0), DeviceIdentity(1)],
        FailingReadsOfARange::Every,
    );
    h.pool.devices = Some(devices);
    ranges.extend(root_slots(7..=7, FailingReadsOfARange::Every));
    ranges.extend(records(7..=7, FailingReadsOfARange::Every));
    let outcome = run_c331_history(h.pool, ranges);
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Mounted { chosen: (2, 6), effective: (2, 6), row_txg: 7, read_stage: ReadStageSettled::OnTheFirstRead { first_read } } if first_read.witness.witnessed_journal_counter == 6),
        "{:?}",
        outcome.mount
    );
    assert!(
        !outcome.final_roots.contains(&(2, 7)),
        "C 的根槽被实例 3 的写行盖掉：{:?}",
        outcome.final_roots
    );
    assert_eq!(outcome.final_read_only, Some((3, 9, true)));
    assert_eq!(outcome.final_writable, Ok((3, 9)));
}

/// 四个系统配置槽只在第 `nth` 次读上坏（第 1 次 `choose_system_configuration`，第 2 次核盘表本盘设备号，第 3 次第一遍判 N-配置 的见证读，
/// 第 4 次重读那一遍的见证读，第 5 次起在读阶段之后），实例 2 的全部根与记录持续读不出，到新实例挂载交回为止。
fn every_system_configuration_slot_failing_only_on_the_nth_read(tag: &str, nth: u64) -> Outcome {
    let mut h = history(tag);
    assert_history_shape(&h);
    let devices = h.pool.reopen_recorded();
    let mut ranges = system_configuration_slots(
        &devices,
        false,
        &[DeviceIdentity(0), DeviceIdentity(1)],
        FailingReadsOfARange::OnlyTheNth(nth),
    );
    h.pool.devices = Some(devices);
    ranges.extend(every_root_and_record_of_the_second_instance(
        &h,
        FailingReadsOfARange::Every,
    ));
    eprintln!(
        "C data unit locations: {:?}",
        h.c.data_pointers
            .iter()
            .flat_map(|p| p.locations)
            .map(|l| (l.device.0, l.slot.0))
            .collect::<Vec<_>>()
    );
    eprintln!(
        "C root tree table / instance table: {:?} / {:?}",
        h.c.root
            .tree_table
            .locations
            .iter()
            .map(|l| (l.device.0, l.slot.0))
            .collect::<Vec<_>>(),
        h.c.root
            .instance_table
            .locations
            .iter()
            .map(|l| (l.device.0, l.slot.0))
            .collect::<Vec<_>>()
    );
    let outcome = run_c331_history(h.pool, ranges);
    if let Some(e) = &outcome.e {
        eprintln!(
            "E data unit locations: {:?}",
            e.data_pointers
                .iter()
                .flat_map(|p| p.locations)
                .map(|l| (l.device.0, l.slot.0))
                .collect::<Vec<_>>()
        );
        eprintln!(
            "E root tree table / instance table: {:?} / {:?}",
            e.root
                .tree_table
                .locations
                .iter()
                .map(|l| (l.device.0, l.slot.0))
                .collect::<Vec<_>>(),
            e.root
                .instance_table
                .locations
                .iter()
                .map(|l| (l.device.0, l.slot.0))
                .collect::<Vec<_>>()
        );
    }
    outcome
}

/// C331 本身（乙-配置续 罩不住的一形）：第一遍的见证读四槽全坏 ⇒ c_见证 = 0、NothingWitnessed、判假，不重读，从 A 可写挂载；
/// 实例 3 写行 4、暖机 5、E 6 确认返回；C（2, 7）的根槽没被盖。撤故障重开：择新择 C，E 被压过。
#[test]
fn every_system_configuration_slot_unreadable_on_the_first_witness_read_lets_a_hidden_root_override_the_new_instance_acknowledged_write(
) {
    let outcome =
        every_system_configuration_slot_failing_only_on_the_nth_read("c331-witness-read-fails", 3);
    assert_eq!(
        (0..4)
            .map(|index| outcome.unreadable.reads_of_range(index))
            .max(),
        Some(outcome.unreadable.reads_of_range(0)),
        "四槽读的次数相同"
    );
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Mounted { chosen: (1, 3), effective: (1, 3), row_txg: 4, read_stage: ReadStageSettled::OnTheFirstRead { first_read } }
            if first_read.witness == NewerPublishWitness { witnessed_journal_counter: 0, comparison: WitnessedCounterComparison::NothingWitnessed }),
        "{:?}",
        outcome.mount
    );
    let e = outcome.e.as_ref().expect("E 确认返回");
    assert_eq!((e.root.instance.0, e.root.checkpoint_txg.0), (3, 6));
    assert!(
        outcome.final_roots.contains(&(2, 7)),
        "C 的根槽还在：{:?}",
        outcome.final_roots
    );
    assert!(
        matches!(outcome.final_read_only, Some((2, 7, false))),
        "撤故障之后只读择 C，读不回 E：{:?}",
        outcome.final_read_only
    );
    assert_ne!(outcome.final_writable, Ok((3, 6)), "可写挂载也择不到 E");
    drop(outcome.pool);
}

/// 同一形的第二种造法：第一遍见证读得出（c_见证 = 7，判真），重读那一遍的见证读四槽全坏 ⇒ 重读判 NothingWitnessed、判假，
/// 按重读那一遍往下走——所选那一版仍是 A，判据翻面只因为见证值没读到。结局同上：E 被 C 压过。
#[test]
fn every_system_configuration_slot_unreadable_on_the_reread_witness_read_flips_the_judgement_and_lets_a_hidden_root_override_the_new_instance_write(
) {
    let outcome = every_system_configuration_slot_failing_only_on_the_nth_read(
        "c331-reread-witness-read-fails",
        4,
    );
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Mounted { chosen: (1, 3), effective: (1, 3), row_txg: 4, read_stage: ReadStageSettled::OnTheOneReread { first_read, reread } }
            if first_read.witness.witnessed_journal_counter == 7
                && first_read.selected_version == reread.selected_version
                && reread.witness == NewerPublishWitness { witnessed_journal_counter: 0, comparison: WitnessedCounterComparison::NothingWitnessed }),
        "{:?}",
        outcome.mount
    );
    assert!(
        matches!(outcome.final_read_only, Some((2, 7, false))),
        "{:?}",
        outcome.final_read_only
    );
    assert_ne!(outcome.final_writable, Ok((3, 6)));
}

/// 推翻条件的造法：同一组故障只把坏的那一次挪到读阶段之后（第 5 次）⇒ 两遍见证都读到 7，拒可写。
#[test]
fn every_system_configuration_slot_unreadable_only_after_the_read_stage_still_refuses() {
    let outcome = every_system_configuration_slot_failing_only_on_the_nth_read(
        "c331-config-fault-after-read-stage",
        5,
    );
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Refused(text) if *text == witness_refusal((1, 3), 3, 7)),
        "{:?}",
        outcome.mount
    );
    assert_eq!(outcome.final_writable, Ok((2, 7)));
}

/// 取号之后的第一个非系统配置写报块设备错（造「取号写落盘之后、写行之前崩溃」）。
struct FailingWritesOutsideTheSystemConfiguration<Inner: BlockDevice> {
    inner: Inner,
}

impl<Inner: BlockDevice> BlockDevice for FailingWritesOutsideTheSystemConfiguration<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
        if offset.0 >= 2 * spacing {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "取号之后的写失败（用例造的崩溃点）",
            )));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 两次故障不同时：第 k 次挂载时两块盘见证 C 的那一槽暂时读不出（C 的根读得出、择根择 C），取号写按「读得出的最大世代号 + 1」
/// 落回那一槽、写进见证值 6，把见证 7 抹掉；写行之前崩溃。第 k + 1 次挂载时 C 的根与记录暂时读不出 ⇒ 见证 6 = B 的末条，判假，
/// 从 B 可写挂载，C（已确认）丢；E 活下来。
#[test]
fn an_acquisition_that_cannot_read_the_newest_system_configuration_slots_overwrites_them_with_an_older_tail_and_a_later_mount_loses_the_acknowledged_publish(
) {
    let mut h = history("c331-acquisition-erases-the-witness");
    assert_history_shape(&h);
    let devices = h.pool.reopen_recorded();
    let newest_slots = system_configuration_slots(
        &devices,
        true,
        &[DeviceIdentity(0), DeviceIdentity(1)],
        FailingReadsOfARange::Every,
    );
    let unreadable =
        SharedUnreadableRanges::new(newest_slots, UnreadableRangeReadBack::DeviceError);
    let mut wrapped = with_unreadable_ranges(
        devices
            .into_iter()
            .map(|(identity, inner)| {
                (
                    identity,
                    FailingWritesOutsideTheSystemConfiguration { inner },
                )
            })
            .collect(),
        &unreadable,
    );
    let crashed = mount_writable(&parameters(), &mut wrapped);
    eprintln!(
        "mount k (crashes after the acquisition): {}",
        describe(&crashed)
    );
    assert!(crashed.is_err(), "写行那一步失败 = 取号之后崩溃");
    h.pool.devices = Some(
        without_unreadable_ranges(wrapped)
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    let devices = h.pool.reopen_recorded();
    let tails_after: Vec<u64> = [DeviceIdentity(0), DeviceIdentity(1)]
        .iter()
        .flat_map(|device| {
            singlefs_core::recovery::verified_system_configuration_slots(
                &devices,
                *device,
                u64::from(parameters().geometry.fixed_structure_slot_spacing),
                &parameters().filesystem_identifier,
            )
        })
        .map(|slot| slot.quantities.journal_tail)
        .collect();
    eprintln!("system configuration tails after the crashed acquisition: {tails_after:?}");
    assert!(
        tails_after.iter().all(|tail| *tail < 7),
        "见证 C 的 tail 7 被抹掉：{tails_after:?}"
    );
    h.pool.devices = Some(devices);
    let mut ranges = root_slots(7..=7, FailingReadsOfARange::Every);
    ranges.extend(records(7..=7, FailingReadsOfARange::Every));
    let outcome = run_c331_history(h.pool, ranges);
    assert!(
        matches!(&outcome.mount, NewInstanceMount::Mounted { chosen: (2, 6), effective: (2, 6), read_stage: ReadStageSettled::OnTheFirstRead { first_read }, .. } if first_read.witness.witnessed_journal_counter == 6),
        "{:?}",
        outcome.mount
    );
    assert!(
        !outcome.final_roots.contains(&(2, 7)),
        "{:?}",
        outcome.final_roots
    );
    assert!(
        matches!(outcome.final_read_only, Some((_, _, true))),
        "E 活下来：{:?}",
        outcome.final_read_only
    );
}
