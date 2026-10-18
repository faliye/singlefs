//! C794 盘上隔离位第一轮攻方腿的原型（只在攻方草稿副本里编、跑；读法开关是副本里打的环境变量补丁，见 `patches/`）。
//! 盘是内存稀疏盘；流都是这里自己造的；崩溃状态经 `enumerate_crash_states_versions`（每一段都展开）判。
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, CrashPointReplaySegmentExpansion,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_versions, CrashPointReplayItem,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
    INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, PublishedVersion, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const IMAGE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"C794-ATTACK-PRB\0";
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const DATA_SPAN: u64 = 2;
const STATE_CAP: u64 = 1_000_000;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
    }
}

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 13 + seed) % 241).expect("小于 256"))
        .collect()
}

/// 读拒绝：某个偏移的读先报错 `remaining` 次（盘上字节不动）。
type Refusal = Arc<Mutex<Option<(u64, u64)>>>;

struct ProbeDevice {
    inner: SparseBlockDevice,
    refusal: Refusal,
}

impl BlockDevice for ProbeDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let mut refusal = self.refusal.lock().expect("锁");
        if let Some((refused_offset, remaining)) = refusal.as_mut() {
            if offset.0 == *refused_offset && *remaining > 0 {
                *remaining -= 1;
                return Err(injected_block_device_error("读"));
            }
        }
        drop(refusal);
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
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
    fn journal_record_offsets_hint(&self, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

type Device = RecordingBlockDevice<ProbeDevice>;

struct Pool {
    devices: Vec<(DeviceIdentity, Device)>,
    stream: SharedStream,
    refusals: Vec<Refusal>,
    allocator: PoolAllocator,
    output: TransactionOutput,
    versions: Vec<PublishedVersion>,
}

impl Pool {
    fn build() -> Self {
        let parameters = parameters();
        let stream = SharedStream::retaining_contents();
        let mut refusals = Vec::new();
        let mut devices: Vec<(DeviceIdentity, Device)> = Vec::new();
        for number in 0..2u32 {
            let refusal: Refusal = Arc::new(Mutex::new(None));
            refusals.push(refusal.clone());
            let inner = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            devices.push((
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(DeviceIdentity(number), ProbeDevice { inner, refusal }, stream.clone()),
            ));
        }
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
            Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
        );
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
        let content = content_of(3000, 1);
        let output = {
            let mut writer = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut writer).expect("取号");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile { content: &content, write_time_seconds: WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("新池新建文件 A")
        };
        let versions = vec![PublishedVersion {
            instance: output.root.instance,
            checkpoint_txg: output.root.checkpoint_txg,
            content,
        }];
        Pool { devices, stream, refusals, allocator, output, versions }
    }

    fn overwrite(&mut self, seed: usize) {
        let parameters = parameters();
        let content = content_of(2500 + seed * 7, seed);
        let next = {
            let mut writer = PoolWriter::new(&parameters, &mut self.devices);
            publish_overwrite(
                &mut writer,
                &mut self.allocator,
                &self.output,
                FirstFile { content: &content, write_time_seconds: WRITE_TIME_SECONDS + 60 * seed as u64 },
                InstanceGeneration(1),
            )
            .expect("覆盖写")
        };
        self.versions.push(PublishedVersion { instance: next.root.instance, checkpoint_txg: next.root.checkpoint_txg, content });
        self.output = next;
    }

    fn roll_back_to(&mut self, txg: u64) -> Result<(), String> {
        let parameters = parameters();
        let target_content = self
            .versions
            .iter()
            .find(|version| version.checkpoint_txg == CheckpointTxg(txg))
            .expect("目标版本在表里")
            .content
            .clone();
        roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(txg) },
        )
        .map_err(|refusal| format!("{refusal:?}"))?;
        self.versions.push(PublishedVersion { instance: self.output.root.instance, checkpoint_txg: self.output.root.checkpoint_txg, content: target_content });
        Ok(())
    }

    fn refuse_reads_at(&self, slot: SlotNumber, times: u64) {
        for refusal in &self.refusals {
            *refusal.lock().expect("锁") = Some((slot.to_device_offset().0, times));
        }
    }
    fn stop_refusing(&self) {
        for refusal in &self.refusals {
            *refusal.lock().expect("锁") = None;
        }
    }

    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self.devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().inner.image.clone())).collect(),
            device_size_in_bytes: IMAGE_BYTES,
        }
    }

    fn records_of(&self, slot: SlotNumber) -> String {
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|device| match self.allocator.record_for(device, slot) {
                Some(record) => format!("盘{}(已释放={},隔离位={},代={})", device.0, record.is_released, record.is_quarantined, record.generation.0),
                None => format!("盘{}(无记录)", device.0),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn verdict_line(image: &MemoryPool) -> String {
    let verdicts = check_pool_image(image);
    let violated: Vec<&str> = verdicts.iter().filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_))).map(|(name, _)| *name).collect();
    let pick = |name: &str| verdicts.iter().find(|(invariant, _)| *invariant == name).map(|(_, verdict)| match verdict {
        InvariantVerdict::Violated(detail) => format!("违例[{}]", detail.chars().take(160).collect::<String>()),
        other => format!("{other:?}").chars().take(60).collect::<String>(),
    }).unwrap_or_default();
    format!("violated={violated:?} I-1.1={} I-3.1={} I-3.11={}", pick("I-1.1"), pick("I-3.1"), pick("I-3.11"))
}

fn cold_recovery_line(image: &MemoryPool, expected: &[u8]) -> String {
    let report = recover(image, JournalPolicy::Consult);
    match &report.outcome {
        singlefs_core::recovery::RecoveryOutcome::FileRead { root, content } => format!("cold=FileRead root={root:?} content_matches_expected={}", content == expected),
        other => format!("cold={other:?}").chars().take(400).collect(),
    }
}

/// 瞬时隔离：A 的数据单元两块盘那一份的读各先报错两次，B 隔离它（两份都完好）。交回 A 的数据槽。
fn transient_quarantine(pool: &mut Pool) -> SlotNumber {
    let data_slot = pool.output.data_pointers[0].locations[0].slot;
    pool.refuse_reads_at(data_slot, 2);
    pool.overwrite(2);
    pool.stop_refusing();
    assert_eq!(pool.output.quarantined_after_release_checksum_mismatch.len(), 2, "前提：B 隔离了两份");
    data_slot
}

fn scenario_q1() {
    // 写者缺陷造出已释放 + 隔离位同时置着的记录（C794_WRITE_BOTH_BITS=1 让编码器照写两位）。
    let mut pool = Pool::build();
    let data_slot = pool.output.data_pointers[0].locations[0].slot;
    pool.overwrite(2); // B 照常释放 A 的数据单元
    println!("Q1 B 之后 A 数据槽 {}: {}", data_slot.0, pool.records_of(data_slot));
    std::env::set_var("C794_WRITE_BOTH_BITS", "1");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        pool.allocator.c794_set_quarantine_flag(device, data_slot, true);
    }
    pool.overwrite(3); // C 写出带两位的记录
    std::env::remove_var("C794_WRITE_BOTH_BITS");
    println!("Q1 C 之后 A 数据槽 {}: {}", data_slot.0, pool.records_of(data_slot));
    let image = pool.image();
    println!("Q1 checker: {}", verdict_line(&image));
    let expected = pool.versions.last().expect("C").content.clone();
    println!("Q1 {}", cold_recovery_line(&image, &expected));
}

fn scenario_q2_image() {
    // 瞬时隔离 B；C；回退到 A（重新引用隔离单元）；D 覆盖写换下它、写者漏了这一次释放；E 落盘。
    let keeps_bit = std::env::var("C794_ROLLBACK_KEEPS_BIT").as_deref() == Ok("1");
    let mut pool = Pool::build();
    let data_slot = transient_quarantine(&mut pool);
    pool.overwrite(3);
    pool.roll_back_to(3).expect("回退到 A");
    println!("Q2 keeps_bit={keeps_bit} 回退之后 槽 {}: {}", data_slot.0, pool.records_of(data_slot));
    println!("Q2 回退之后 checker: {}", verdict_line(&pool.image()));
    let after_rollback = pool.output.clone();
    assert_eq!(after_rollback.data_pointers[0].locations[0].slot, data_slot, "回退那一版引用隔离单元");
    pool.overwrite(4); // D：换下 U
    println!("Q2 D 照常释放之后 槽 {}: {}", data_slot.0, pool.records_of(data_slot));
    // 写者漏释放：D 那一次该释放 U 却没有——记录停在 D 之前的样子（仍分配、代 3、隔离位照 D 之前）。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        pool.allocator.resurrect_released_record(device, Placement { slot: data_slot, span: DATA_SPAN }, CheckpointTxg(3));
        pool.allocator.c794_set_quarantine_flag(device, data_slot, keeps_bit);
    }
    pool.overwrite(5); // E 把漏释放的账写盘
    println!("Q2 漏释放之后 槽 {}: {}", data_slot.0, pool.records_of(data_slot));
    println!("Q2 漏释放之后 checker: {}", verdict_line(&pool.image()));
}

fn scenario_q5_image() {
    // 写者缺陷：隔离只在盘 1 那条记录上置位，盘 0 那条仍分配不带位（各盘不对称）。
    let mut pool = Pool::build();
    let data_slot = transient_quarantine(&mut pool);
    pool.allocator.c794_set_quarantine_flag(DeviceIdentity(0), data_slot, false);
    pool.overwrite(3);
    println!("Q5 读法={:?} 槽 {}: {}", std::env::var("C794_EXEMPT_READING").ok(), data_slot.0, pool.records_of(data_slot));
    println!("Q5 checker: {}", verdict_line(&pool.image()));
    // 对照：写者两块盘都置位（实现今天的样子）。
    let mut symmetric = Pool::build();
    let symmetric_slot = transient_quarantine(&mut symmetric);
    symmetric.overwrite(3);
    println!("Q5 对照（两盘都置位） 槽 {}: {}", symmetric_slot.0, symmetric.records_of(symmetric_slot));
    println!("Q5 对照 checker: {}", verdict_line(&symmetric.image()));
}

fn scenario_q4(corrupt: bool) {
    // 写者漏释放（不带位）：B 该释放 A 的数据单元 U 却没有；C；（可选）盘 1 上 U 那一份坏掉；回退到 A。
    let mut pool = Pool::build();
    let data_slot = pool.output.data_pointers[0].locations[0].slot;
    pool.overwrite(2);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        pool.allocator.resurrect_released_record(device, Placement { slot: data_slot, span: DATA_SPAN }, CheckpointTxg(3));
    }
    pool.overwrite(3);
    println!("Q4 corrupt={corrupt} 读法 bit={:?} 回退前 槽 {}: {}", std::env::var("C794_ISOLATED_BY_BIT").ok(), data_slot.0, pool.records_of(data_slot));
    println!("Q4 回退前 checker: {}", verdict_line(&pool.image()));
    if corrupt {
        let (_, device) = pool.devices.iter_mut().find(|(identity, _)| *identity == DeviceIdentity(1)).expect("盘 1");
        let offset = DeviceOffsetInBytes(data_slot.to_device_offset().0 + 4096);
        let mut byte = device.wrapped_device().inner.image.read(offset, 512);
        byte[0] ^= 0xff;
        device.wrapped_device_mut().inner.image.write(offset, &byte);
    }
    match pool.roll_back_to(3) {
        Ok(()) => {
            println!("Q4 回退做成 槽 {}: {}", data_slot.0, pool.records_of(data_slot));
            println!("Q4 回退之后 checker: {}", verdict_line(&pool.image()));
            let expected = pool.versions.last().expect("回退").content.clone();
            println!("Q4 回退之后 {}", cold_recovery_line(&pool.image(), &expected));
        }
        Err(refusal) => println!("Q4 回退被拒：{}", refusal.chars().take(300).collect::<String>()),
    }
}

/// 崩溃枚举：`which` 选一段历史，只展开它要判的那一次发布（基线 = 那次发布之前的整条流）。
fn crash_flow(which: &str) -> (MemoryPool, Vec<singlefs_harness::memory_pool::RetainedWrite>, Vec<Vec<usize>>, usize, Vec<PublishedVersion>) {
    let mut pool = Pool::build();
    let prefix_length;
    match which {
        "quarantine-publish" => {
            prefix_length = pool.stream.retained_operations().len();
            transient_quarantine(&mut pool);
        }
        "rollback-publish" => {
            transient_quarantine(&mut pool);
            pool.overwrite(3);
            prefix_length = pool.stream.retained_operations().len();
            pool.roll_back_to(3).expect("回退到 A");
        }
        "after-rollback-overwrite" => {
            transient_quarantine(&mut pool);
            pool.overwrite(3);
            pool.roll_back_to(3).expect("回退到 A");
            prefix_length = pool.stream.retained_operations().len();
            pool.overwrite(4);
        }
        other => panic!("认不得的流 {other}"),
    }
    let operations = pool.stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..prefix_length]);
    let (writes, segments) = writes_and_segments(&operations[prefix_length..], &geometry());
    let root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("这次发布有根槽写");
    (base, writes, segments, root_index, pool.versions)
}

fn every_proper_subset(_: usize, _: &[usize]) -> CrashPointReplaySegmentExpansion {
    CrashPointReplaySegmentExpansion::EveryProperSubset
}

fn count(which: &str) -> u64 {
    let (base, writes, segments, root_index, _) = crash_flow(which);
    let states = crash_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &every_proper_subset);
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!("COUNT flow={which} writes={} segments={sizes:?} root_index={root_index} states={states}", writes.len());
    states
}

fn enumerate(which: &str, label: &str) {
    let states = count(which);
    assert!(states <= STATE_CAP, "这一段 {states} 个状态越过约 10^6，不跑");
    let (base, writes, segments, root_index, versions) = crash_flow(which);
    let item_name = format!("c794-attack-{which}-{label}");
    let tally = enumerate_crash_states_versions(
        CrashPointReplayItem { name: &item_name, setting_file: "crates/singlefs-checker-tier/src/bin/c794_attack_probe.rs" },
        &base,
        &writes,
        &segments,
        root_index,
        &versions,
    )
    .judged()
    .expect("单机两段跑完交回整份结果");
    let violated: BTreeMap<&str, u64> = tally.checker_violated_states.iter().filter(|(_, count)| **count > 0).map(|(name, count)| (*name, *count)).collect();
    println!(
        "ENUM flow={which} label={label} states={} oracle_violations={} failed_states={} file_read={} checker_violated={violated:?}",
        tally.states, tally.violations, tally.failed_states, tally.file_read_states
    );
    for (name, detail) in &tally.checker_first_violation {
        if violated.contains_key(name) {
            println!("ENUM_FIRST label={label} {name}: {}", detail.chars().take(300).collect::<String>());
        }
    }
    if let Some(first) = &tally.first_violation {
        println!("ENUM_FIRST_ORACLE label={label}: {}", first.chars().take(300).collect::<String>());
    }
}

/// 估时：只判开头 `how_many` 个状态（同一份枚举、同一个判器），报挂钟。
fn sample(which: &str, how_many: u64) {
    let (base, writes, segments, root_index, versions) = crash_flow(which);
    let started = std::time::Instant::now();
    let mut judged = 0u64;
    let item_name = format!("c794-attack-sample-{which}");
    let _ = singlefs_checker_tier::crash_state_enumeration::sample_crash_states_on_the_cpu(
        CrashPointReplayItem { name: &item_name, setting_file: "crates/singlefs-checker-tier/src/bin/c794_attack_probe.rs" },
        &base,
        &writes,
        &segments,
        root_index,
        &versions,
        &every_proper_subset,
        0..how_many,
        &mut |_, _, _| judged += 1,
    );
    println!("SAMPLE flow={which} judged={judged} seconds={:.1}", started.elapsed().as_secs_f64());
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    match arguments.get(1).map(String::as_str) {
        Some("q1") => scenario_q1(),
        Some("q2-image") => scenario_q2_image(),
        Some("q5-image") => scenario_q5_image(),
        Some("q4") => scenario_q4(arguments.get(2).map(String::as_str) == Some("corrupt")),
        Some("count") => {
            count(&arguments[2]);
        }
        Some("enumerate") => enumerate(&arguments[2], &arguments[3]),
        Some("sample") => sample(&arguments[2], arguments[3].parse().expect("个数")),
        other => panic!("认不得的子命令 {other:?}"),
    }
}
