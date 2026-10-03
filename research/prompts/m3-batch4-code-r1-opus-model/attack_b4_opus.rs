//! m3-batch4-code-r1 攻方腿的原型（只在草稿副本里跑，不进仓库的测试）：C2 四处分歧与 C5、C1 的造镜像与小枚举。
//! 盘一律是内存稀疏盘；每个起点池只建一次，之后每段历史从内存里那一份拷。

mod common;

use std::collections::BTreeMap;

use common::{file_content, memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{attack_replay_outcome_for_differential, check_pool_image};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, MemoryPool, SparseBlockDevice,
};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Device = RecordingBlockDevice<SparseBlockDevice>;

fn devices_of(pool: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, Device)> {
    pool.devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect()
}

fn empty_pool() -> MemoryPool {
    MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES)
}

fn geometry_of(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 起点池：mkfs → 可写挂载（取号、写行、暖机）→ 第一个文件（A）。交回池、分配器与 A。
struct Started {
    pool: MemoryPool,
    allocator: singlefs_core::allocator::PoolAllocator,
    output: TransactionOutput,
    instance: InstanceGeneration,
}

fn start(parameters: &MakeFilesystemParameters) -> Started {
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&empty_pool(), &stream);
    make_filesystem(parameters, &mut devices).expect("mkfs");
    let mounted = mount_writable_with_space_admission(
        parameters,
        &mut devices,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
    .expect("可写挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator;
    let content = file_content();
    let output = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        match &mounted.current {
            PoolVersion::WithoutFile(current) => publish_first_file(
                &mut writer,
                &mut allocator,
                &current.root,
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &current.record_bytes,
            )
            .expect("第一个文件"),
            PoolVersion::WithFile(_) => panic!("mkfs 之后是没有文件的一版"),
        }
    };
    Started { pool: memory_pool_of_sparse_devices(&devices), allocator, output, instance }
}

/// 在 `pool` 的副本上接一次覆盖写，交回之后的池、这一次发布的录制写表与段、新输出。
fn overwrite_recorded(
    parameters: &MakeFilesystemParameters,
    pool: &MemoryPool,
    allocator: &mut singlefs_core::allocator::PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
    content: &[u8],
) -> (MemoryPool, Vec<singlefs_harness::memory_pool::RetainedWrite>, Vec<Vec<usize>>, TransactionOutput) {
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(pool, &stream);
    let output = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            allocator,
            previous,
            FirstFile { content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            instance,
        )
        .expect("覆盖写")
    };
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry_of(parameters));
    (memory_pool_of_sparse_devices(&devices), writes, segments, output)
}

fn violated(pool: &dyn singlefs_checker::image::ImageReader) -> Vec<(&'static str, String)> {
    check_pool_image(pool)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((name, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn verdict_of(pool: &dyn singlefs_checker::image::ImageReader, invariant: &str) -> InvariantVerdict {
    check_pool_image(pool)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .expect("清单里有")
        .1
}

fn core_outcome(pool: &dyn PoolReader) -> String {
    let report = recover(pool, JournalPolicy::Consult);
    let effective = report
        .effective_root
        .map(|(instance, txg)| format!("({},{})", instance.0, txg.0))
        .unwrap_or_else(|| "none".to_string());
    let outcome = match report.outcome {
        RecoveryOutcome::FileRead { content, .. } => format!("file {} bytes", content.len()),
        RecoveryOutcome::NoFile { .. } => "no file".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed {failure:?}"),
    };
    format!("{effective} {outcome}")
}

fn content_read(pool: &dyn PoolReader) -> Option<Option<Vec<u8>>> {
    match recover(pool, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => Some(Some(content)),
        RecoveryOutcome::NoFile { .. } => Some(None),
        RecoveryOutcome::Failed { .. } => None,
    }
}

fn read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(pool, DeviceIdentity(device), DeviceOffsetInBytes(offset), length).expect("读")
}

fn write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    pool.devices.get_mut(&DeviceIdentity(device)).expect("盘").write(DeviceOffsetInBytes(offset), bytes);
}

const SYSCONF_IN_FLIGHT_LIMIT: usize = 345;
const SYSCONF_WORST_CASE: usize = 349;
const SYSCONF_SAFETY_FACTOR: usize = 357;
const SYSCONF_SLOT_SPACING: usize = 429;

fn sysconf_slots(pool: &MemoryPool) -> Vec<(u32, u64)> {
    let mut slots = Vec::new();
    for device in [0u32, 1] {
        let slot_zero = read(pool, device, 0, 4096);
        let spacing = u64::from(u32::from_le_bytes(
            slot_zero[SYSCONF_SLOT_SPACING..SYSCONF_SLOT_SPACING + 4].try_into().expect("4"),
        ));
        slots.push((device, 0));
        slots.push((device, spacing));
    }
    slots
}

fn mutate_sysconf(pool: &mut MemoryPool, device: u32, offset: u64, change: impl Fn(&mut Vec<u8>)) {
    let mut bytes = read(pool, device, offset, 4096);
    change(&mut bytes);
    let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 155);
    bytes[155..187].copy_from_slice(&digest);
    write(pool, device, offset, &bytes);
}

/// 把 F 改成 `factor`、在飞上限与最坏占用按它重算（三个等式按槽里的 F 都成立）。`slots` 为空时改全部四槽。
fn with_safety_factor(clean: &MemoryPool, factor: u32, only: Option<(u32, u64)>) -> MemoryPool {
    let mut pool = clean.clone();
    let ring_records = parameters().geometry.journal_ring_bytes / 4096;
    let targets = match only {
        Some(one) => vec![one],
        None => sysconf_slots(&pool),
    };
    for (device, offset) in targets {
        mutate_sysconf(&mut pool, device, offset, |bytes| {
            let limit = ring_records / u64::from(factor);
            bytes[SYSCONF_SAFETY_FACTOR..SYSCONF_SAFETY_FACTOR + 4].copy_from_slice(&factor.to_le_bytes());
            bytes[SYSCONF_IN_FLIGHT_LIMIT..SYSCONF_IN_FLIGHT_LIMIT + 4]
                .copy_from_slice(&u32::try_from(limit).expect("4 字节").to_le_bytes());
            bytes[SYSCONF_WORST_CASE..SYSCONF_WORST_CASE + 8].copy_from_slice(&(limit * 4096).to_le_bytes());
        });
    }
    pool
}

fn writable_mount_on(parameters: &MakeFilesystemParameters, pool: &MemoryPool) -> Result<MemoryPool, String> {
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(pool, &stream);
    match mount_writable_with_space_admission(parameters, &mut devices, SpaceAdmission::SkippedByTheTestOnlySwitch) {
        Ok(_) => Ok(memory_pool_of_sparse_devices(&devices)),
        Err(error) => Err(format!("{error:?}")),
    }
}

fn not_applicable_count(pool: &dyn singlefs_checker::image::ImageReader) -> usize {
    check_pool_image(pool)
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::NotApplicable(_)))
        .count()
}

/// C2-1：F ≠ 3 而三个字段按槽里的 F 自洽的镜像，checker 与 core 各判什么。
#[test]
fn c2_1_safety_factor_other_than_the_format_constant() {
    let params = parameters();
    let clean = start(&params).pool;
    println!("C2-1 clean: checker violated={:?} core={}", violated(&clean), core_outcome(&clean));
    let spacing = u64::from(params.geometry.fixed_structure_slot_spacing);
    let cases: Vec<(&str, MemoryPool)> = vec![
        ("F=2 four slots", with_safety_factor(&clean, 2, None)),
        ("F=4 four slots", with_safety_factor(&clean, 4, None)),
        ("F=2 dev1 slot1 only", with_safety_factor(&clean, 2, Some((1, spacing)))),
        ("F=1 dev0 slot0 only", with_safety_factor(&clean, 1, Some((0, 0)))),
    ];
    for (name, image) in cases {
        println!(
            "C2-1 {name}: checker violated={:?} I-8.1={:?} I-7.13={:?} not_applicable={} | core recover={} | core mount_writable={:?}",
            violated(&image),
            verdict_of(&image, "I-8.1"),
            verdict_of(&image, "I-7.13"),
            not_applicable_count(&image),
            core_outcome(&image),
            writable_mount_on(&params, &image).err()
        );
    }
}

fn record_bytes_at(pool: &MemoryPool, device: u32, counter: u64) -> Vec<u8> {
    read(pool, device, record_offset(counter, parameters().geometry.journal_ring_bytes).0, 4096)
}

fn parse_record(bytes: &[u8]) -> Option<JournalRecord> {
    JournalRecord::parse(
        bytes,
        singlefs_core::unit::unit_filesystem_identifier(&parameters().filesystem_identifier),
    )
}

/// 「B 的记录两盘都落了、根槽没落」：A 之后覆盖写一次，写表里根槽 FUA 之前的写全落。
fn image_with_b_record_but_not_its_root() -> (MemoryPool, u64, u64) {
    let params = parameters();
    let mut started = start(&params);
    let content: Vec<u8> = (0..4100usize).map(|index| u8::try_from((index * 7 + 3) % 253).expect("小")).collect();
    let (_, writes, _, output_b) =
        overwrite_recorded(&params, &started.pool, &mut started.allocator, &started.output, started.instance, &content);
    let root_index = writes
        .iter()
        .position(|write| write.kind == singlefs_harness::segments::StepKind::RootRecordFua)
        .expect("有根槽写");
    let mut image = started.pool.clone();
    image.apply_writes(&writes[..root_index]);
    let txg_b = output_b.root.checkpoint_txg.0;
    let counter_b = (1..64)
        .find(|counter| parse_record(&record_bytes_at(&image, 1, *counter)).is_some_and(|record| record.checkpoint_txg.0 == txg_b))
        .expect("B 那条记录");
    (image, txg_b, counter_b)
}

fn corrupt_new_tree_table_checksums(pool: &mut MemoryPool, device: u32, counter: u64) {
    let offset = record_offset(counter, parameters().geometry.journal_ring_bytes).0;
    let mut record = read(pool, device, offset, 4096);
    for checksum_offset in [99 + 50 + 10, 99 + 64 + 10] {
        let value = u32::from_le_bytes(record[checksum_offset..checksum_offset + 4].try_into().expect("4"));
        record[checksum_offset..checksum_offset + 4].copy_from_slice(&value.wrapping_add(1).to_le_bytes());
    }
    let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
    record[46..78].copy_from_slice(&digest);
    write(pool, device, offset, &record);
}

/// C2-2：两块盘同一计数器坐着不同实例的记录。盘 0 那一格换成实例 2 的一条记录（系统配置先取号到实例 2，I-7.7 不先红），
/// 盘 1 那一格仍是 B（实例 1）。
#[test]
fn c2_2_same_counter_different_instances_on_the_two_devices() {
    let params = parameters();
    let (b_record_only, txg_b, counter_b) = image_with_b_record_but_not_its_root();
    println!(
        "C2-2 B-record-only (txg {txg_b}, counter {counter_b}): checker replay={} I-4.1={:?} violated={:?} | core={}",
        attack_replay_outcome_for_differential(&b_record_only),
        verdict_of(&b_record_only, "I-4.1"),
        violated(&b_record_only),
        core_outcome(&b_record_only)
    );
    let mut forged = {
        let stream = SharedStream::retaining_contents();
        let mut devices = devices_of(&b_record_only, &stream);
        {
            let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号");
            assert_eq!(instance, InstanceGeneration(2));
        }
        memory_pool_of_sparse_devices(&devices)
    };
    let record_b = parse_record(&record_bytes_at(&forged, 1, counter_b)).expect("盘 1 上是 B");
    let mut other = record_b.clone();
    other.instance = InstanceGeneration(2);
    other.checkpoint_txg = CheckpointTxg(txg_b + 1);
    other.back_chain = 0;
    other.transaction = 1;
    other.is_commit = true;
    other.place_in_publish = JournalRecordPlaceInPublish::LastRecordOfThePublish;
    other.named = Vec::new();
    write(&mut forged, 0, record_offset(counter_b, params.geometry.journal_ring_bytes).0, &other.to_bytes());
    println!(
        "C2-2 forged (dev0 counter {counter_b} = instance 2 txg {}; dev1 = B): checker replay={} I-4.1={:?} violated={:?} | core={}",
        txg_b + 1,
        attack_replay_outcome_for_differential(&forged),
        verdict_of(&forged, "I-4.1"),
        violated(&forged),
        core_outcome(&forged)
    );
    let mut forged_and_b_unreadable = forged.clone();
    corrupt_new_tree_table_checksums(&mut forged_and_b_unreadable, 1, counter_b);
    println!(
        "C2-2 forged + dev1 B new tree table unreadable: checker replay={} I-4.1={:?} violated={:?} | core={}",
        attack_replay_outcome_for_differential(&forged_and_b_unreadable),
        verdict_of(&forged_and_b_unreadable, "I-4.1"),
        violated(&forged_and_b_unreadable),
        core_outcome(&forged_and_b_unreadable)
    );
    let mut control = b_record_only.clone();
    corrupt_new_tree_table_checksums(&mut control, 0, counter_b);
    corrupt_new_tree_table_checksums(&mut control, 1, counter_b);
    println!(
        "C2-2 control (no forging, both copies of B unreadable tree table): checker replay={} I-4.1={:?} violated={:?} | core={}",
        attack_replay_outcome_for_differential(&control),
        verdict_of(&control, "I-4.1"),
        violated(&control),
        core_outcome(&control)
    );
}

/// 层 0 的枚举域：更早的段整段持久、当前段取每个非空子集、更晚的段一个都没持久，外加一个都没持久的那一态。
fn for_each_crash_state(
    base: &MemoryPool,
    writes: &[singlefs_harness::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    mut visit: impl FnMut(&MemoryPool, String),
) {
    visit(base, "none".to_string());
    let mut running = base.clone();
    let skip_unit_only = std::env::var("ATTACK_SKIP_UNIT_ONLY_SEGMENTS").is_ok_and(|value| value == "1");
    for (segment_index, segment) in segments.iter().enumerate() {
        let length = segment.len();
        if skip_unit_only
            && segment.iter().all(|index| writes[*index].kind == singlefs_harness::segments::StepKind::UnitWrite)
        {
            for write_index in segment {
                running.apply_writes(std::slice::from_ref(&writes[*write_index]));
            }
            visit(&running, format!("seg{segment_index}/whole-unit-segment"));
            continue;
        }
        assert!(length < 20, "段长 {length}：子集太多");
        for mask in 1u64..(1u64 << length) {
            let mut image = running.clone();
            for (within, write_index) in segment.iter().enumerate() {
                if (mask >> within) & 1 == 1 {
                    image.apply_writes(std::slice::from_ref(&writes[*write_index]));
                }
            }
            visit(&image, format!("seg{segment_index}/mask{mask:b}"));
        }
        for write_index in segment {
            running.apply_writes(std::slice::from_ref(&writes[*write_index]));
        }
    }
}

#[derive(Default)]
struct Tally {
    counts: BTreeMap<String, u64>,
    examples: BTreeMap<String, String>,
}

impl Tally {
    fn note(&mut self, key: String, example: impl FnOnce() -> String) {
        let count = self.counts.entry(key.clone()).or_insert(0);
        *count += 1;
        if *count == 1 {
            self.examples.insert(key, example());
        }
    }
    fn print(&self, title: &str) {
        for (key, count) in &self.counts {
            println!("{title} {key}: {count}  例: {}", self.examples.get(key).map_or("", String::as_str));
        }
    }
}

/// 一个崩溃状态上：池级 checker 判红的每一条、I-4.1 判成什么、R-独 与 core 的 effective_root 一不一致。
fn judge_state(stage: &str, image: &MemoryPool, label: &str, tally: &mut Tally) {
    tally.note(format!("{stage} states"), || label.to_string());
    for (invariant, detail) in violated(image) {
        tally.note(format!("{stage} RED {invariant}"), || format!("{label}: {detail}"));
    }
    let i41 = match verdict_of(image, "I-4.1") {
        InvariantVerdict::Holds => "holds".to_string(),
        InvariantVerdict::Violated(_) => "violated".to_string(),
        InvariantVerdict::NotApplicable(reason) => format!("n/a {}", reason.chars().take(18).collect::<String>()),
    };
    tally.note(format!("{stage} I-4.1 {i41}"), || label.to_string());
    let checker = attack_replay_outcome_for_differential(image);
    let core = core_outcome(image);
    let core_root = core.split(' ').next().unwrap_or("").to_string();
    if checker == core_root {
        tally.note(format!("{stage} differential agree"), || format!("{label}: {checker}"));
    } else {
        tally.note(format!("{stage} differential DIFFER"), || format!("{label}: checker={checker} core={core}"));
    }
}

/// 一次发布的录制写表：从第一条记录那一段起的每个崩溃状态（第一次崩溃）上判，再在它上面起一次可写挂载、录下挂载那一段，
/// 挂载那一段的每个崩溃状态（二次崩溃）上：池级 checker、R-独 对 core、再挂一次之后读出的内容与没崩过的那一次比（R-换）。
fn enumerate_publish_suffix_and_the_mount_after_each_state(
    params: &MakeFilesystemParameters,
    pool_before_the_publish: &MemoryPool,
    writes: &[singlefs_harness::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    title: &str,
) {
    let first_record_segment = segments
        .iter()
        .position(|segment| segment.iter().any(|index| writes[*index].kind == singlefs_harness::segments::StepKind::JournalRecord))
        .expect("这次发布有记录");
    let mut base = pool_before_the_publish.clone();
    for segment in &segments[..first_record_segment] {
        for index in segment {
            base.apply_writes(std::slice::from_ref(&writes[*index]));
        }
    }
    let suffix = &segments[first_record_segment..];
    let kinds = |segments: &[Vec<usize>], writes: &[singlefs_harness::memory_pool::RetainedWrite]| -> Vec<Vec<String>> {
        segments.iter().map(|segment| segment.iter().map(|index| format!("{:?}", writes[*index].kind)).collect()).collect()
    };
    println!("{title} 发布段（从第一条记录那一段起）{:?}，全量 {}", kinds(suffix, writes), closed_form_state_count(suffix));
    let mut first_tally = Tally::default();
    let mut second_tally = Tally::default();
    let mut second_states_total = 0u64;
    let limit_first: usize = std::env::var("ATTACK_FIRST_LIMIT").ok().and_then(|text| text.parse().ok()).unwrap_or(usize::MAX);
    let mut first_seen = 0usize;
    for_each_crash_state(&base, writes, suffix, |first, label| {
        first_seen += 1;
        if first_seen > limit_first {
            return;
        }
        judge_state("first", first, &label, &mut first_tally);
        let stream = SharedStream::retaining_contents();
        let mut devices = devices_of(first, &stream);
        if let Err(error) = mount_writable_with_space_admission(params, &mut devices, SpaceAdmission::SkippedByTheTestOnlySwitch) {
            first_tally.note("first writable mount REFUSED".to_string(), || format!("{label}: {error:?}"));
            return;
        }
        let after = memory_pool_of_sparse_devices(&devices);
        let once = content_read(&after);
        let (writes_m, segments_m) = writes_and_segments(&stream.retained_operations(), &geometry_of(params));
        second_states_total += closed_form_state_count(&segments_m);
        println!("{title} first {label}: 挂载那一段 {:?}，全量 {}", kinds(&segments_m, &writes_m), closed_form_state_count(&segments_m));
        for_each_crash_state(first, &writes_m, &segments_m, |second, label2| {
            let full_label = format!("first {label} second {label2}");
            judge_state("second", second, &full_label, &mut second_tally);
            match writable_mount_on(params, second) {
                Err(error) => second_tally.note("R-换 remount REFUSED".to_string(), || format!("{full_label}: {error}")),
                Ok(remounted) => {
                    let twice = content_read(&remounted);
                    let key = match (&once, &twice) {
                        (None, _) => "R-换 once read nothing (not judged)".to_string(),
                        (Some(a), Some(b)) if a == b => "R-换 same content".to_string(),
                        _ => "R-换 DIFFERENT content".to_string(),
                    };
                    second_tally.note(key, || format!("{full_label}: once={:?} twice={:?}", once.as_ref().map(|c| c.as_ref().map(Vec::len)), twice.as_ref().map(|c| c.as_ref().map(Vec::len))));
                }
            }
        });
    });
    println!("{title} 第一次崩溃 {first_seen} 个状态、挂载那一段的闭式全量合计 {second_states_total}");
    first_tally.print(&format!("{title} FIRST"));
    second_tally.print(&format!("{title} SECOND"));
}

fn content_of(seed: usize, length: usize) -> Vec<u8> {
    (0..length).map(|index| u8::try_from((index * 7 + seed) % 253).expect("小")).collect()
}

/// 历史一：A 之后覆盖写 B（实例 1）。
#[test]
fn c2_3_c2_4_c1_history_one_overwrite_b_after_a() {
    let params = parameters();
    let mut started = start(&params);
    let (_, writes, segments, _) = overwrite_recorded(
        &params, &started.pool, &mut started.allocator, &started.output, started.instance, &content_of(3, 4100),
    );
    enumerate_publish_suffix_and_the_mount_after_each_state(&params, &started.pool, &writes, &segments, "H1");
}

/// 历史二：A 之后覆盖写 B 崩在 B 的根槽之前；可写挂载（实例 2，重放 B、写行、暖机）做完；实例 2 覆盖写 C。
/// 枚举 C 那次发布，C 的每个状态上再起实例 3 的挂载。
#[test]
fn c2_3_c2_4_c1_history_two_overwrite_c_by_instance_two_after_b_was_replayed() {
    let params = parameters();
    let (b_record_only, _, _) = image_with_b_record_but_not_its_root();
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&b_record_only, &stream);
    let mounted = mount_writable_with_space_admission(&params, &mut devices, SpaceAdmission::SkippedByTheTestOnlySwitch)
        .expect("实例 2 的挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator;
    let previous = match mounted.current {
        PoolVersion::WithFile(output) => output,
        PoolVersion::WithoutFile(_) => panic!("B 带文件"),
    };
    let after_mount = memory_pool_of_sparse_devices(&devices);
    let (_, writes, segments, _) =
        overwrite_recorded(&params, &after_mount, &mut allocator, &previous, instance, &content_of(11, 2500));
    enumerate_publish_suffix_and_the_mount_after_each_state(&params, &after_mount, &writes, &segments, "H2");
}

/// C1：mkfs 收下的几何上，新池新建文件之后的镜像 checker 红不红（I-7.5、I-8.1、I-7.13 这几条按声明几何判）；mkfs 拒的记下成员。
#[test]
fn c1_every_geometry_mkfs_accepts_holds_on_the_new_pool_file_creation_image() {
    let mut tally = Tally::default();
    for physical_block_size in [512u32, 1024, 2048, 4096] {
        for minimum_input_output_bytes in [0u32, 512, 1024, 2048, 3072, 4096, 6144, 8192, 12288, 16384] {
            for journal_ring_bytes in [1u64 << 20, 3 << 20, singlefs_format::JOURNAL_RING_DEFAULT_BYTES] {
                let mut params = parameters();
                params.geometry.physical_block_size = physical_block_size;
                params.geometry.minimum_input_output_bytes = minimum_input_output_bytes;
                let width = u64::from(physical_block_size.max(minimum_input_output_bytes)).max(1);
                params.geometry.fixed_structure_slot_spacing =
                    u32::try_from(4096u64.div_ceil(width) * width).expect("槽距");
                params.geometry.journal_ring_bytes = journal_ring_bytes;
                let label = format!(
                    "pbs {physical_block_size} io_min {minimum_input_output_bytes} spacing {} ring {journal_ring_bytes}",
                    params.geometry.fixed_structure_slot_spacing
                );
                let stream = SharedStream::retaining_contents();
                let mut devices: Vec<(DeviceIdentity, Device)> = [0u32, 1]
                    .into_iter()
                    .map(|number| {
                        let identity = DeviceIdentity(number);
                        (
                            identity,
                            RecordingBlockDevice::with_shared_stream(
                                identity,
                                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(physical_block_size)),
                                stream.clone(),
                            ),
                        )
                    })
                    .collect();
                if let Err(error) = make_filesystem(&params, &mut devices) {
                    let member = format!("{error:?}");
                    let short: String = member.split(['{', '(']).next().unwrap_or("").to_string();
                    tally.note(format!("mkfs refused {short}"), || format!("{label}: {member}"));
                    continue;
                }
                let after_mkfs = memory_pool_of_sparse_devices(&devices);
                for (invariant, detail) in violated(&after_mkfs) {
                    tally.note(format!("after mkfs RED {invariant}"), || format!("{label}: {detail}"));
                }
                let mounted = match mount_writable_with_space_admission(&params, &mut devices, SpaceAdmission::SkippedByTheTestOnlySwitch) {
                    Ok(mounted) => mounted,
                    Err(error) => {
                        tally.note("mount refused after mkfs".to_string(), || format!("{label}: {error:?}"));
                        continue;
                    }
                };
                let instance = mounted.output.instance;
                let mut allocator = mounted.allocator;
                let content = file_content();
                let published = {
                    let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
                    match &mounted.current {
                        PoolVersion::WithoutFile(current) => publish_first_file(
                            &mut writer,
                            &mut allocator,
                            &current.root,
                            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                            instance,
                            &current.record_bytes,
                        )
                        .map(|_| ()),
                        PoolVersion::WithFile(_) => panic!("mkfs 之后没有文件"),
                    }
                };
                if let Err(error) = published {
                    tally.note("first file refused".to_string(), || format!("{label}: {error:?}"));
                    continue;
                }
                let image = memory_pool_of_sparse_devices(&devices);
                tally.note("accepted and published".to_string(), || label.clone());
                for (invariant, detail) in violated(&image) {
                    tally.note(format!("after first file RED {invariant}"), || format!("{label}: {detail}"));
                }
                let core = core_outcome(&image);
                if !core.contains("file 3000 bytes") {
                    tally.note("core did not read the file".to_string(), || format!("{label}: {core}"));
                }
            }
        }
    }
    tally.print("C1-geometry");
}

/// C2-3 的平局：mkfs 刚写完时根环里只有第 0 代的三条根，它们逐字节是不是相同（不同的话 checker 取最后一条、core 取第一条）。
#[test]
fn c2_3_seed_roots_written_by_mkfs_are_byte_identical() {
    let params = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&empty_pool(), &stream);
    make_filesystem(&params, &mut devices).expect("mkfs");
    let image = memory_pool_of_sparse_devices(&devices);
    let (_, chosen) = singlefs_checker::image::chosen_system_configurations(&image)
        .into_iter()
        .find(|(_, chosen)| chosen.is_some())
        .expect("择到系统配置");
    let (_, geometry) = chosen.expect("有");
    let roots = singlefs_checker::image::valid_roots(&image, &geometry);
    let keys: Vec<(u64, u64, u32, u64)> = roots.iter().map(|(region, slot, root)| (*region, *slot, root.instance, root.checkpoint_txg)).collect();
    let distinct: std::collections::BTreeSet<Vec<u8>> = roots.iter().map(|(_, _, root)| root.record_bytes.clone()).collect();
    println!("C2-3 seed roots after mkfs: {keys:?} distinct record bytes {}", distinct.len());
    println!("C2-3 seed roots after mkfs: checker replay={} core={}", attack_replay_outcome_for_differential(&image), core_outcome(&image));
}
