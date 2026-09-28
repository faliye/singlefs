//! unreadable-at-mount-r1 云端攻方腿的原型（只在仓副本里跑）：C331 的候选（今天、甲、乙、丙）在「见证读 / 根 / 记录分别在第几次读坏、
//! 持续还是暂时」的历史上，挂载拒不拒、丢不丢已确认的写。每段历史打一行 `name=c331grid …`，不做断言：几份副本的行按键对齐比。
//!
//! 取样：起点状态每个只建一次池（文件镜像建完立刻转成内存稀疏盘），之后每段历史从内存里那一份拷；
//! 挂载之后由用户决定的那几步（新实例确认几次写）放开扫 1..=K，逐次快照、各判一次。
mod common;

use common::{
    build_pool, crash_state_devices, parameters, publish_overwrite_in_process,
    with_unreadable_ranges,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration,
};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 新实例挂载之后第 i 次覆盖写的内容（长度各不相同，读回时按它的长度读）。
fn new_instance_content(publish_index: usize) -> Vec<u8> {
    content_of(3700 + 37 * publish_index, 29 + publish_index)
}

const MOST_PUBLISHES_AFTER_THE_MOUNT: usize = 3;

/// 起点状态：盘上的样子，与判丢写要的事实。
struct Prefix {
    name: &'static str,
    image: MemoryPool,
    /// 最新一次确认返回的发布 (实例, txg)。挂载选中的版本 txg 比它小就是丢了它。
    newest_acknowledged: (u32, u64),
    /// 实例 2 发过的全部根的 txg（含没确认的）与它们的记录计数器区间。
    second_instance_root_txgs: std::ops::RangeInclusive<u64>,
    second_instance_record_counters: std::ops::RangeInclusive<u64>,
    /// 最新那条根（可能没确认）的 txg、它那条记录的计数器、它的第一个数据单元盘 0 那一份的偏移。
    newest_root_txg: u64,
    newest_record_counter: u64,
    newest_data_unit_offset_on_device_0: u64,
    /// 每块盘两槽：(槽偏移, 世代号)；读不出或自证不过的槽世代号记 None。
    system_configuration_slots: Vec<(DeviceIdentity, u64, Option<u64>)>,
}

fn slot_spacing() -> u64 {
    u64::from(parameters().geometry.fixed_structure_slot_spacing)
}

fn system_configuration_slots_of(image: &MemoryPool) -> Vec<(DeviceIdentity, u64, Option<u64>)> {
    let mut out = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let verified = singlefs_core::recovery::verified_system_configuration_slots(
            image,
            device,
            slot_spacing(),
            &parameters().filesystem_identifier,
        );
        for position in 0..2u64 {
            let generation = verified
                .iter()
                .map(|slot| slot.quantities.slot_generation)
                .find(|generation| generation % 2 == position);
            out.push((device, position * slot_spacing(), generation));
        }
    }
    out
}

fn tails_of(image: &MemoryPool) -> Vec<(u32, u64, u64)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .iter()
        .flat_map(|device| {
            singlefs_core::recovery::verified_system_configuration_slots(
                image,
                *device,
                slot_spacing(),
                &parameters().filesystem_identifier,
            )
            .into_iter()
            .map(move |slot| (device.0, slot.quantities.slot_generation, slot.quantities.journal_tail))
        })
        .collect()
}

/// A（实例 1，txg 3）→ 重开，实例 2 写行 4、暖机 5，再发 `publishes_of_the_second_instance` 次覆盖写（txg 6 起），全部确认返回。
fn standard_history(
    tag: &str,
    publishes_of_the_second_instance: usize,
) -> (common::BuiltPool, Vec<TransactionOutput>) {
    let mut pool = build_pool(tag);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("实例 2 可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    let mut outputs = Vec::new();
    for index in 0..publishes_of_the_second_instance {
        let previous = pool.output.clone();
        let next = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of(4100 - 100 * index, 3 + index),
            FIXED_WRITE_TIME_SECONDS + 60 * (1 + u64::try_from(index).expect("小")),
            InstanceGeneration(2),
        )
        .expect("实例 2 覆盖写");
        pool.output = next.clone();
        outputs.push(next);
    }
    (pool, outputs)
}

fn prefix_from(
    name: &'static str,
    image: MemoryPool,
    newest_acknowledged: &TransactionOutput,
    newest: &TransactionOutput,
) -> Prefix {
    let system_configuration_slots = system_configuration_slots_of(&image);
    Prefix {
        name,
        image,
        newest_acknowledged: (
            newest_acknowledged.root.instance.0,
            newest_acknowledged.root.checkpoint_txg.0,
        ),
        second_instance_root_txgs: 4..=newest.root.checkpoint_txg.0,
        second_instance_record_counters: 4..=newest.record.counter,
        newest_root_txg: newest.root.checkpoint_txg.0,
        newest_record_counter: newest.record.counter,
        newest_data_unit_offset_on_device_0: newest.data_pointers[0]
            .locations
            .iter()
            .find(|location| location.device == DeviceIdentity(0))
            .expect("盘 0 那一份")
            .slot
            .to_device_offset()
            .0,
        system_configuration_slots,
    }
}

/// 同一次写撕裂时「新旧都读不出」的字节（层 0 的算法，`singlefs_checker_tier::crash::torn_image_of_in_place_overwrite` 原样手抄一份：
/// 不同的那一截前一半新、后一半旧）。
fn torn(old: &[u8], new: &[u8]) -> Vec<u8> {
    let differs = |index: &usize| old[*index] != new[*index];
    let Some(first) = (0..old.len()).find(differs) else {
        return new.to_vec();
    };
    let last = (0..old.len()).rev().find(differs).expect("有第一个就有最后一个");
    let first_old = first + (last - first + 1) / 2;
    let mut bytes = new.to_vec();
    bytes[first_old..].copy_from_slice(&old[first_old..]);
    bytes
}

#[derive(Clone, Copy, Debug)]
enum RotationOfD {
    /// 盘 0 那一写撕裂，盘 1 那一写没持久。
    TornOnDevice0OnlyAndDevice1NotPersisted,
    /// 两块盘那一写都撕裂。
    TornOnBoth,
    /// 盘 0 撕裂，盘 1 持久了。
    TornOnDevice0AndDevice1Persisted,
}

/// 标准历史之后实例 2 再发 D（txg 8）：根 FUA 落盘、记录落盘，轮换系统配置那一写（每块盘一写，同一段）按 `rotation` 撕裂或没持久，
/// 之后崩溃——D 没确认返回。零读故障就是层 0 枚举得到的合法崩溃状态（系统配置槽写是原地覆写、长于一个扇区，取第三态）。
fn torn_rotation_prefix(name: &'static str, rotation: RotationOfD) -> Prefix {
    let (mut pool, outputs) = standard_history(name, 2);
    let c = outputs.last().expect("C").clone();
    let before = pool.memory_pool();
    let previous = pool.output.clone();
    let d = publish_overwrite_in_process(
        &mut pool,
        &previous,
        &content_of(3300, 17),
        FIXED_WRITE_TIME_SECONDS + 180,
        InstanceGeneration(2),
    )
    .expect("D 在进程里发完");
    let mut image = pool.memory_pool();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let newest_generation_before = singlefs_core::recovery::verified_system_configuration_slots(
            &before,
            device,
            slot_spacing(),
            &parameters().filesystem_identifier,
        )
        .iter()
        .map(|slot| slot.quantities.slot_generation)
        .max()
        .expect("有");
        let offset = DeviceOffsetInBytes(((newest_generation_before + 1) % 2) * slot_spacing());
        let old = before.devices[&device].read(offset, slot_bytes);
        let new = image.devices[&device].read(offset, slot_bytes);
        assert_ne!(old, new, "D 的轮换写了这一槽");
        let landed = match (rotation, device.0) {
            (_, 0) => torn(&old, &new),
            (RotationOfD::TornOnDevice0OnlyAndDevice1NotPersisted, _) => old,
            (RotationOfD::TornOnBoth, _) => torn(&old, &new),
            (RotationOfD::TornOnDevice0AndDevice1Persisted, _) => new,
        };
        image.devices.get_mut(&device).expect("盘").write(offset, &landed);
    }
    eprintln!("prefix {name}: tails after the torn rotation {:?}", tails_of(&image));
    prefix_from(name, image, &c, &d)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotSet {
    None,
    AllFour,
    NewestOnBoth,
    NewestOnDevice0,
    OlderOnDevice0,
    BothOnDevice1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HiddenSet {
    None,
    /// 实例 2 的全部根与全部记录（两份）。
    AllRootsAndRecordsOfTheSecondInstance,
    /// 实例 2 的全部根，记录读得出。
    AllRootsOfTheSecondInstance,
    /// 最新那条根与它的记录（两份）。
    NewestRootAndRecord,
    /// 最新那条根与它的数据单元盘 0 那一份（记录读得出，重放验点名单元不过）。
    NewestRootAndOneDataCopy,
}

#[derive(Clone, Copy, Debug)]
struct Scenario {
    slots: SlotSet,
    slot_timing: FailingReadsOfARange,
    hidden: HiddenSet,
    hidden_timing: FailingReadsOfARange,
    read_back: UnreadableRangeReadBack,
}

fn slot_ranges(prefix: &Prefix, set: SlotSet, failing: FailingReadsOfARange) -> Vec<UnreadableRange> {
    let newest_generation_of = |device: DeviceIdentity| {
        prefix
            .system_configuration_slots
            .iter()
            .filter(|(candidate, _, _)| *candidate == device)
            .filter_map(|(_, _, generation)| *generation)
            .max()
            .expect("每块盘至少一槽自证过")
    };
    let chosen: Vec<(DeviceIdentity, u64)> = prefix
        .system_configuration_slots
        .iter()
        .filter(|(device, offset, _)| {
            let newest_offset = (newest_generation_of(*device) % 2) * slot_spacing();
            match set {
                SlotSet::None => false,
                SlotSet::AllFour => true,
                SlotSet::NewestOnBoth => *offset == newest_offset,
                SlotSet::NewestOnDevice0 => device.0 == 0 && *offset == newest_offset,
                SlotSet::OlderOnDevice0 => device.0 == 0 && *offset != newest_offset,
                SlotSet::BothOnDevice1 => device.0 == 1,
            }
        })
        .map(|(device, offset, _)| (*device, *offset))
        .collect();
    chosen
        .into_iter()
        .map(|(device, offset)| UnreadableRange {
            device,
            offset_in_bytes: offset,
            length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
            failing_reads: failing,
        })
        .collect()
}

fn root_ranges(txgs: std::ops::RangeInclusive<u64>, failing: FailingReadsOfARange) -> Vec<UnreadableRange> {
    txgs.map(|txg| common::unreadable_root_slot_of(CheckpointTxg(txg), failing))
        .collect()
}

fn record_ranges(
    counters: std::ops::RangeInclusive<u64>,
    failing: FailingReadsOfARange,
) -> Vec<UnreadableRange> {
    counters
        .flat_map(|counter| {
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(move |device| UnreadableRange {
                    device,
                    offset_in_bytes: record_offset(counter, parameters().geometry.journal_ring_bytes).0,
                    length_in_bytes: JOURNAL_RECORD_BYTES,
                    failing_reads: failing,
                })
        })
        .collect()
}

fn hidden_ranges(prefix: &Prefix, set: HiddenSet, failing: FailingReadsOfARange) -> Vec<UnreadableRange> {
    match set {
        HiddenSet::None => Vec::new(),
        HiddenSet::AllRootsAndRecordsOfTheSecondInstance => {
            let mut ranges = root_ranges(prefix.second_instance_root_txgs.clone(), failing);
            ranges.extend(record_ranges(prefix.second_instance_record_counters.clone(), failing));
            ranges
        }
        HiddenSet::AllRootsOfTheSecondInstance => {
            root_ranges(prefix.second_instance_root_txgs.clone(), failing)
        }
        HiddenSet::NewestRootAndRecord => {
            let mut ranges = root_ranges(prefix.newest_root_txg..=prefix.newest_root_txg, failing);
            ranges.extend(record_ranges(
                prefix.newest_record_counter..=prefix.newest_record_counter,
                failing,
            ));
            ranges
        }
        HiddenSet::NewestRootAndOneDataCopy => {
            let mut ranges = root_ranges(prefix.newest_root_txg..=prefix.newest_root_txg, failing);
            ranges.push(UnreadableRange {
                device: DeviceIdentity(0),
                offset_in_bytes: prefix.newest_data_unit_offset_on_device_0,
                length_in_bytes: singlefs_format::DATA_UNIT_BYTES,
                failing_reads: failing,
            });
            ranges
        }
    }
}

/// 一次「崩溃之后冷重开」的只读挂载：择到的 (实例, txg) 与读回的文件（按 `length` 读）。
fn read_only_view(image: &MemoryPool, length: usize) -> Option<(u32, u64, Vec<u8>)> {
    let read_only = mount_read_only(image).ok()?;
    let bytes = read_only
        .mounted
        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
        .ok()
        .and_then(|file| {
            file.read_at(image, FileOffsetInBytes(0), u64::try_from(length).expect("长度"))
                .ok()
        })
        .map(|out| out.bytes)
        .unwrap_or_default();
    Some((
        read_only.effective_root.instance.0,
        read_only.effective_root.checkpoint_txg.0,
        bytes,
    ))
}

type FaultyDevices = Vec<(
    DeviceIdentity,
    common::DeviceWithUnreadableRanges<
        singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
    >,
)>;

/// 这几块盘此刻的整份镜像（「这一刻崩溃」）。
fn snapshot_of(devices: &FaultyDevices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner_device().wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    }
}

/// 一段历史跑完的判定。
fn run_scenario(prefix: &Prefix, scenario: &Scenario) -> String {
    let mut ranges = slot_ranges(prefix, scenario.slots, scenario.slot_timing);
    ranges.extend(hidden_ranges(prefix, scenario.hidden, scenario.hidden_timing));
    let unreadable = SharedUnreadableRanges::new(ranges, scenario.read_back);
    let stream = SharedStream::new();
    let mut devices = with_unreadable_ranges(
        crash_state_devices(&prefix.image, &[], &[], &stream),
        &unreadable,
    );
    let mounted = mount_writable(&parameters(), &mut devices);
    let key = format!(
        "prefix={} slots={:?} slot_timing={:?} hidden={:?} hidden_timing={:?} read_back={:?}",
        prefix.name,
        scenario.slots,
        scenario.slot_timing,
        scenario.hidden,
        scenario.hidden_timing,
        scenario.read_back
    );
    let mut mounted: Mounted = match mounted {
        Err(error) => {
            let short = format!("{error:?}");
            let short: String = short.chars().take(90).collect();
            return format!("name=c331grid {key} verdict=refused mount=Err({short})");
        }
        Ok(mounted) => mounted,
    };
    unreadable.lift();
    let effective = (
        mounted.output.effective_root.instance.0,
        mounted.output.effective_root.checkpoint_txg.0,
    );
    let older_acknowledged_lost = effective.1 < prefix.newest_acknowledged.1;
    let mut per_publish = Vec::new();
    let mut new_instance_lost = false;
    for publish_index in 1..=MOST_PUBLISHES_AFTER_THE_MOUNT {
        let previous = mounted.current.file_version().expect("带文件").clone();
        let content = new_instance_content(publish_index);
        let params = parameters();
        let published = {
            let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut mounted.allocator,
                &previous,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
                },
                mounted.output.instance,
            )
        };
        let published = match published {
            Ok(output) => output,
            Err(error) => {
                per_publish.push(format!("k{publish_index}=publish_err({error:?})"));
                break;
            }
        };
        mounted.current = singlefs_core::transaction::PoolVersion::WithFile(published.clone());
        // 这次写确认返回之后立刻崩溃：拿此刻的镜像冷重开。
        let image = snapshot_of(&devices);
        let view = read_only_view(&image, content.len());
        let survived = matches!(&view, Some((instance, txg, bytes))
            if (*instance, *txg) == (published.root.instance.0, published.root.checkpoint_txg.0) && *bytes == content);
        if !survived {
            new_instance_lost = true;
        }
        per_publish.push(format!(
            "k{publish_index}=({},{})->{}",
            published.root.instance.0,
            published.root.checkpoint_txg.0,
            match &view {
                Some((instance, txg, _)) if survived => format!("kept({instance},{txg})"),
                Some((instance, txg, _)) => format!("LOST_final({instance},{txg})"),
                None => "LOST_read_only_refused".to_string(),
            }
        ));
    }
    let verdict = match (older_acknowledged_lost, new_instance_lost) {
        (false, false) => "writable_no_loss",
        (true, false) => "writable_LOST_older_acknowledged",
        (false, true) => "writable_LOST_new_instance",
        (true, true) => "writable_LOST_both",
    };
    format!(
        "name=c331grid {key} verdict={verdict} effective=({},{}) newest_acknowledged=({},{}) read_stage={} {}",
        effective.0,
        effective.1,
        prefix.newest_acknowledged.0,
        prefix.newest_acknowledged.1,
        short_read_stage(&mounted.output.rereads.read_stage),
        per_publish.join(" ")
    )
}

fn short_reading(reading: &singlefs_core::mount::SelectedVersionAgainstTheWitness) -> String {
    use singlefs_core::mount::WitnessedCounterComparison;
    let comparison = match reading.witness.comparison {
        WitnessedCounterComparison::NothingWitnessed => "nothing".to_string(),
        WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
            selected_version_last_record_counter,
        } => format!("last_record={selected_version_last_record_counter}"),
        WitnessedCounterComparison::AgainstTheRecordAtTheWitnessedCounter { record } => {
            format!("record_at_witness=({},{})", record.instance.0, record.checkpoint_txg.0)
        }
        WitnessedCounterComparison::Undecidable => "undecidable".to_string(),
    };
    format!(
        "sel({},{})w{}:{comparison}",
        reading.selected_version.instance.0,
        reading.selected_version.checkpoint_txg.0,
        reading.witness.witnessed_journal_counter
    )
}

fn short_read_stage(settled: &singlefs_core::mount::ReadStageSettled) -> String {
    use singlefs_core::mount::ReadStageSettled;
    match settled {
        ReadStageSettled::OnTheFirstRead { first_read } => format!("first[{}]", short_reading(first_read)),
        ReadStageSettled::OnTheOneReread { first_read, reread } => {
            format!("reread[{}|{}]", short_reading(first_read), short_reading(reread))
        }
    }
}

fn standard_prefix(name: &'static str, publishes_of_the_second_instance: usize) -> Prefix {
    let (pool, outputs) = standard_history(name, publishes_of_the_second_instance);
    let newest = outputs.last().expect("至少一次").clone();
    let image = pool.memory_pool();
    eprintln!(
        "prefix {name}: newest ({},{}) jsn {} tails {:?}",
        newest.root.instance.0,
        newest.root.checkpoint_txg.0,
        newest.record.counter,
        tails_of(&image)
    );
    prefix_from(name, image, &newest, &newest)
}

static STANDARD: std::sync::OnceLock<PrefixCell> = std::sync::OnceLock::new();
static LONG: std::sync::OnceLock<PrefixCell> = std::sync::OnceLock::new();
static TORN_DEVICE_0: std::sync::OnceLock<PrefixCell> = std::sync::OnceLock::new();
static TORN_BOTH: std::sync::OnceLock<PrefixCell> = std::sync::OnceLock::new();
static TORN_DEVICE_0_DEVICE_1_PERSISTED: std::sync::OnceLock<PrefixCell> = std::sync::OnceLock::new();

/// `Prefix` 里只有内存镜像与数：跨测试线程共享，只读。
struct PrefixCell(Prefix);

#[derive(Clone, Copy, Debug)]
enum PrefixName {
    Standard,
    Long,
    TornDevice0,
    TornBoth,
    TornDevice0Device1Persisted,
}

fn prefix(name: PrefixName) -> &'static Prefix {
    let cell = match name {
        PrefixName::Standard => STANDARD.get_or_init(|| PrefixCell(standard_prefix("standard", 2))),
        PrefixName::Long => LONG.get_or_init(|| PrefixCell(standard_prefix("long", 4))),
        PrefixName::TornDevice0 => TORN_DEVICE_0.get_or_init(|| {
            PrefixCell(torn_rotation_prefix(
                "torn_d0",
                RotationOfD::TornOnDevice0OnlyAndDevice1NotPersisted,
            ))
        }),
        PrefixName::TornBoth => TORN_BOTH
            .get_or_init(|| PrefixCell(torn_rotation_prefix("torn_both", RotationOfD::TornOnBoth))),
        PrefixName::TornDevice0Device1Persisted => TORN_DEVICE_0_DEVICE_1_PERSISTED.get_or_init(|| {
            PrefixCell(torn_rotation_prefix(
                "torn_d0_d1_persisted",
                RotationOfD::TornOnDevice0AndDevice1Persisted,
            ))
        }),
    };
    &cell.0
}

const SLOT_TIMINGS: [FailingReadsOfARange; 5] = [
    FailingReadsOfARange::Every,
    FailingReadsOfARange::OnlyTheNth(3),
    FailingReadsOfARange::OnlyTheNth(4),
    FailingReadsOfARange::NthThroughMth(3, 4),
    FailingReadsOfARange::FromTheNthOnward(5),
];

const HIDDEN_TIMINGS: [FailingReadsOfARange; 3] = [
    FailingReadsOfARange::Every,
    FailingReadsOfARange::OnlyTheFirst,
    FailingReadsOfARange::AllButTheNth(2),
];

fn hidden_choices() -> Vec<(HiddenSet, FailingReadsOfARange)> {
    let mut choices = vec![(HiddenSet::None, FailingReadsOfARange::Never)];
    for set in [
        HiddenSet::AllRootsAndRecordsOfTheSecondInstance,
        HiddenSet::AllRootsOfTheSecondInstance,
        HiddenSet::NewestRootAndRecord,
        HiddenSet::NewestRootAndOneDataCopy,
    ] {
        for timing in HIDDEN_TIMINGS {
            choices.push((set, timing));
        }
    }
    choices
}

fn run_grid(prefix_name: PrefixName, slots: SlotSet) {
    let prefix = prefix(prefix_name);
    let slot_timings: Vec<FailingReadsOfARange> = if slots == SlotSet::None {
        vec![FailingReadsOfARange::Never]
    } else {
        SLOT_TIMINGS.to_vec()
    };
    let mut lines = 0usize;
    let started = std::time::Instant::now();
    for slot_timing in slot_timings {
        for (hidden, hidden_timing) in hidden_choices() {
            let read_backs: &[UnreadableRangeReadBack] =
                if slots == SlotSet::None && hidden == HiddenSet::None {
                    &[UnreadableRangeReadBack::DeviceError]
                } else {
                    &[UnreadableRangeReadBack::DeviceError, UnreadableRangeReadBack::Zeros]
                };
            for read_back in read_backs {
                let scenario = Scenario {
                    slots,
                    slot_timing,
                    hidden,
                    hidden_timing,
                    read_back: *read_back,
                };
                eprintln!("{}", run_scenario(prefix, &scenario));
                lines += 1;
            }
        }
    }
    eprintln!(
        "name=c331grid-count prefix={} slots={slots:?} scenarios={lines} seconds={}",
        prefix.name,
        started.elapsed().as_secs()
    );
}

macro_rules! grid_tests {
    ($($test_name:ident => ($prefix:expr, $slots:expr);)*) => {
        $(
            #[test]
            fn $test_name() {
                run_grid($prefix, $slots);
            }
        )*
    };
}

grid_tests! {
    grid_standard_no_slot_fault => (PrefixName::Standard, SlotSet::None);
    grid_standard_all_four => (PrefixName::Standard, SlotSet::AllFour);
    grid_standard_newest_on_both => (PrefixName::Standard, SlotSet::NewestOnBoth);
    grid_standard_newest_on_device_0 => (PrefixName::Standard, SlotSet::NewestOnDevice0);
    grid_standard_older_on_device_0 => (PrefixName::Standard, SlotSet::OlderOnDevice0);
    grid_standard_both_on_device_1 => (PrefixName::Standard, SlotSet::BothOnDevice1);
    grid_long_no_slot_fault => (PrefixName::Long, SlotSet::None);
    grid_long_all_four => (PrefixName::Long, SlotSet::AllFour);
    grid_long_newest_on_both => (PrefixName::Long, SlotSet::NewestOnBoth);
    grid_long_newest_on_device_0 => (PrefixName::Long, SlotSet::NewestOnDevice0);
    grid_long_older_on_device_0 => (PrefixName::Long, SlotSet::OlderOnDevice0);
    grid_long_both_on_device_1 => (PrefixName::Long, SlotSet::BothOnDevice1);
    grid_torn_device_0_no_slot_fault => (PrefixName::TornDevice0, SlotSet::None);
    grid_torn_device_0_newest_on_both => (PrefixName::TornDevice0, SlotSet::NewestOnBoth);
    grid_torn_device_0_newest_on_device_0 => (PrefixName::TornDevice0, SlotSet::NewestOnDevice0);
    grid_torn_both_no_slot_fault => (PrefixName::TornBoth, SlotSet::None);
    grid_torn_both_newest_on_both => (PrefixName::TornBoth, SlotSet::NewestOnBoth);
    grid_torn_device_0_device_1_persisted_no_slot_fault => (PrefixName::TornDevice0Device1Persisted, SlotSet::None);
}

/// 取号之后的第一个非系统配置写报块设备错：造「取号写落盘之后、写行之前崩溃」（V4 第 k 次挂载那一步）。
struct CrashAfterTheAcquisition<Inner: singlefs_core::block_device::BlockDevice> {
    inner: Inner,
}

impl<Inner: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
    for CrashAfterTheAcquisition<Inner>
{
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        if offset.0 >= 2 * slot_spacing() {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("取号之后的写失败（原型造的崩溃点）"),
            ));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// V4：第 k 次挂载时 `slots` 按 `slot_timing` 读坏、取号之后崩溃；第 k + 1 次挂载按 `hidden` 读坏，之后同 `run_scenario`。
fn run_two_mounts(
    slots: SlotSet,
    slot_timing: FailingReadsOfARange,
    read_back: UnreadableRangeReadBack,
    hidden: HiddenSet,
) -> String {
    let base = prefix(PrefixName::Standard);
    let unreadable = SharedUnreadableRanges::new(slot_ranges(base, slots, slot_timing), read_back);
    let stream = SharedStream::new();
    let mut devices = with_unreadable_ranges(
        crash_state_devices(&base.image, &[], &[], &stream)
            .into_iter()
            .map(|(identity, inner)| (identity, CrashAfterTheAcquisition { inner }))
            .collect(),
        &unreadable,
    );
    let mount_k = mount_writable(&parameters(), &mut devices);
    let mount_k_text: String = match &mount_k {
        Ok(_) => "Ok".to_string(),
        Err(error) => format!("{error:?}").chars().take(70).collect(),
    };
    let image_k = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| {
                (*identity, device.inner_device().inner.wrapped_device().image.clone())
            })
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    };
    let tails_k = tails_of(&image_k);
    let after_k = Prefix {
        name: "v4_after_mount_k",
        system_configuration_slots: system_configuration_slots_of(&image_k),
        image: image_k,
        newest_acknowledged: base.newest_acknowledged,
        second_instance_root_txgs: base.second_instance_root_txgs.clone(),
        second_instance_record_counters: base.second_instance_record_counters.clone(),
        newest_root_txg: base.newest_root_txg,
        newest_record_counter: base.newest_record_counter,
        newest_data_unit_offset_on_device_0: base.newest_data_unit_offset_on_device_0,
    };
    let line = run_scenario(
        &after_k,
        &Scenario {
            slots: SlotSet::None,
            slot_timing: FailingReadsOfARange::Never,
            hidden,
            hidden_timing: FailingReadsOfARange::Every,
            read_back: UnreadableRangeReadBack::DeviceError,
        },
    );
    format!(
        "name=c331v4 mount_k_slots={slots:?} mount_k_slot_timing={slot_timing:?} mount_k_read_back={read_back:?} mount_k=[{mount_k_text}] tails_after_k={tails_k:?} || {line}"
    )
}

fn run_v4_grid(slots: SlotSet) {
    let mut lines = 0usize;
    for slot_timing in [
        FailingReadsOfARange::Every,
        FailingReadsOfARange::FromTheNthOnward(5),
        FailingReadsOfARange::FromTheNthOnward(6),
        FailingReadsOfARange::FromTheNthOnward(7),
        FailingReadsOfARange::FromTheNthOnward(8),
        FailingReadsOfARange::NthThroughMth(5, 8),
    ] {
        for read_back in [UnreadableRangeReadBack::DeviceError, UnreadableRangeReadBack::Zeros] {
            for hidden in [
                HiddenSet::NewestRootAndRecord,
                HiddenSet::NewestRootAndOneDataCopy,
                HiddenSet::AllRootsAndRecordsOfTheSecondInstance,
            ] {
                eprintln!("{}", run_two_mounts(slots, slot_timing, read_back, hidden));
                lines += 1;
            }
        }
    }
    eprintln!("name=c331v4-count slots={slots:?} scenarios={lines}");
}

#[test]
fn v4_grid_newest_on_both() {
    run_v4_grid(SlotSet::NewestOnBoth);
}
#[test]
fn v4_grid_all_four() {
    run_v4_grid(SlotSet::AllFour);
}
#[test]
fn v4_grid_newest_on_device_0() {
    run_v4_grid(SlotSet::NewestOnDevice0);
}
#[test]
fn v4_grid_both_on_device_1() {
    run_v4_grid(SlotSet::BothOnDevice1);
}
