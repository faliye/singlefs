
// ════════════ 正推腿探针（Q2 / Q3 / Q5）════════════
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, PROBE_BIT_RECORDS_REFERENCED, PROBE_EXEMPT_BY_UNIT};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::mount::{raise_rollback_floor, ShadowLedger};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::{RecordedOperationKind, RetainedOperation};

fn violated_of(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect()
}

fn counters() -> (u64, u64) {
    PROBE_BIT_RECORDS_REFERENCED.with(|a| (a[0].get(), a[1].get()))
}

fn bits_of(pool: &BuiltPool, slot: SlotNumber) -> String {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| {
            let r: &AllocationRecord = pool.allocator.record_for(device, slot).expect("记录在");
            format!("d{}(released={},bit={},gen={})", device.0, r.is_released, r.is_quarantined, r.generation.0)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn say(label: &str, pool: &BuiltPool, slot: SlotNumber) {
    let image = pool.memory_pool();
    let violated = violated_of(&image);
    let (union, newest) = counters();
    println!(
        "PROBE-R {label}: checker_violated={violated:?} bit_records_referenced(union={union},newest={newest}) records[{}]",
        bits_of(pool, slot)
    );
}

fn raise_floor_to(pool: &mut BuiltPool, floor: u64) {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut current = pool.output.clone();
    raise_rollback_floor(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(floor),
        ShadowLedger::On,
    )
    .expect("抬 F");
    pool.output = current;
}

fn overwrite(pool: &mut BuiltPool, seed: usize, seconds: u64) {
    let previous = pool.output.clone();
    let output = publish_overwrite_in_process(
        pool,
        &previous,
        &(0..2600usize).map(|i| u8::try_from((i * 13 + seed) % 241).expect("u8")).collect::<Vec<_>>(),
        FIXED_WRITE_TIME_SECONDS + seconds,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    pool.output = output;
}


// 瞬时隔离（盘上一个字节都不坏）：与 checker_quarantine_flag_exempts_only_flagged_unreferenced_records.rs 同一个手写故障设备
use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::fault_injection::injected_block_device_error;
use std::cell::Cell;

struct ReadsAtOneOffsetRefusedAFewTimes<Inner: BlockDevice> {
    inner: Inner,
    refused_offset: DeviceOffsetInBytes,
    reads_to_refuse: Cell<u64>,
}

impl<Inner: BlockDevice> BlockDevice for ReadsAtOneOffsetRefusedAFewTimes<Inner> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if offset == self.refused_offset && self.reads_to_refuse.get() > 0 {
            self.reads_to_refuse.set(self.reads_to_refuse.get() - 1);
            return Err(injected_block_device_error("读"));
        }
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
}

/// 新池新建文件 A（txg 3）之后覆盖写 B（txg 4）：A 的数据单元每块盘那一份的读先报错两次，B 隔离它（盘上不动）。
fn transient_isolation_pool(tag: &str) -> (BuiltPool, SlotNumber) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let mut wrapped: Vec<(DeviceIdentity, ReadsAtOneOffsetRefusedAFewTimes<Recorded>)> = pool
        .devices
        .take()
        .expect("镜像还开着")
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                ReadsAtOneOffsetRefusedAFewTimes {
                    inner: device,
                    refused_offset: data_slot.to_device_offset(),
                    reads_to_refuse: Cell::new(2),
                },
            )
        })
        .collect();
    let second = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &first,
            FirstFile {
                content: &second_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("B")
    };
    pool.devices = Some(wrapped.into_iter().map(|(identity, device)| (identity, device.inner)).collect());
    pool.output = second;
    (pool, data_slot)
}

/// 隔离 → 回退到 A（引用隔离单元）→ 覆盖写 C → D E F → 抬 F，在清位 / 不清位两种读法下，与「C 漏释放」写者缺陷叠加的矩阵。
fn run_matrix(no_clear: bool, forgotten_release: bool) {
    if no_clear {
        std::env::set_var("PROBE_NO_CLEAR", "1");
    } else {
        std::env::remove_var("PROBE_NO_CLEAR");
    }
    let tag = format!("probe-matrix-{no_clear}-{forgotten_release}");
    let (mut pool, data_slot) = transient_isolation_pool(&tag);
    let mode = format!("no_clear={no_clear} forgotten_release={forgotten_release}");
    say(&format!("[{mode}] 1 after B (transiently isolated, A still in the ring)"), &pool, data_slot);
    roll_back_to_the_first_version(&mut pool).expect("回退照做");
    say(&format!("[{mode}] 2 after rollback to A (unit referenced again by the newest root)"), &pool, data_slot);
    let before_c: Vec<AllocationRecord> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| *pool.allocator.record_for(device, data_slot).expect("记录"))
        .collect();
    overwrite(&mut pool, 5, 120);
    say(&format!("[{mode}] 3 after overwrite C (releases the unit)"), &pool, data_slot);
    if forgotten_release {
        for record in &before_c {
            pool.allocator.resurrect_released_record(
                record.device,
                singlefs_core::allocator::Placement { slot: data_slot, span: 2 },
                record.generation,
            );
            pool.allocator.probe_replace_record(*record);
        }
        println!("PROBE-R [{mode}] 3b writer defect: both records put back to the pre-C value {:?}", before_c.iter().map(|r| (r.is_released, r.is_quarantined)).collect::<Vec<_>>());
    }
    for (seed, seconds) in [(6, 180), (7, 240), (8, 300)] {
        overwrite(&mut pool, seed, seconds);
    }
    say(&format!("[{mode}] 4 after overwrite D E F"), &pool, data_slot);
    let newest_txg = pool.output.root.checkpoint_txg.0;
    for floor in (4..=newest_txg).rev() {
        let before = pool.allocator.clone();
        let output_before = pool.output.clone();
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut current = pool.output.clone();
        match raise_rollback_floor(&publish_parameters, devices, &mut pool.allocator, &mut current, CheckpointTxg(floor), ShadowLedger::On) {
            Ok(_) => {
                pool.output = current;
                say(&format!("[{mode}] 5 after raising F to {floor} (newest txg {newest_txg})"), &pool, data_slot);
                break;
            }
            Err(error) => {
                println!("PROBE-R [{mode}] raise F to {floor} refused: {error:?}");
                pool.allocator = before;
                pool.output = output_before;
            }
        }
    }
}

#[test]
fn probe_matrix() {
    for no_clear in [false, true] {
        for forgotten_release in [false, true] {
            run_matrix(no_clear, forgotten_release);
        }
    }
    std::env::remove_var("PROBE_NO_CLEAR");
}

// ════════════ 崩溃状态：窗口里每个切点 × 切点前未被屏障定住的写的子集 ════════════
use std::collections::BTreeMap;

struct Window {
    name: &'static str,
    start: usize,
    end: usize,
}

fn crash_images(pool: &BuiltPool, window: &Window, per_cut_cap: usize) -> Vec<(usize, MemoryPool)> {
    let ops: Vec<RetainedOperation> = pool.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
    base.apply(&ops[..window.start]);
    let mut random_state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next_random = move || {
        random_state ^= random_state << 13;
        random_state ^= random_state >> 7;
        random_state ^= random_state << 17;
        random_state
    };
    let mut images = Vec::new();
    for cut in window.start..=window.end {
        let mut forced = Vec::new();
        let mut free = Vec::new();
        for index in window.start..cut {
            let operation = &ops[index].operation;
            if operation.kind == RecordedOperationKind::Barrier {
                continue;
            }
            let barrier_after = ops[index + 1..cut].iter().any(|later| {
                later.operation.kind == RecordedOperationKind::Barrier
                    && later.operation.device == operation.device
            });
            if operation.kind == RecordedOperationKind::WriteForceUnitAccess || barrier_after {
                forced.push(index);
            } else {
                free.push(index);
            }
        }
        let mut masks: Vec<Vec<bool>> = Vec::new();
        if free.len() <= 8 {
            for bits in 0..(1u32 << free.len()) {
                masks.push((0..free.len()).map(|i| bits >> i & 1 == 1).collect());
            }
        } else {
            masks.push(vec![false; free.len()]);
            masks.push(vec![true; free.len()]);
            for skip in 0..free.len() {
                let mut mask = vec![true; free.len()];
                mask[skip] = false;
                masks.push(mask);
            }
            for _ in 0..per_cut_cap {
                masks.push((0..free.len()).map(|_| next_random() & 1 == 1).collect());
            }
        }
        for mask in masks {
            let mut chosen: Vec<usize> = forced.clone();
            chosen.extend(free.iter().zip(&mask).filter(|(_, keep)| **keep).map(|(index, _)| *index));
            chosen.sort_unstable();
            let selected: Vec<RetainedOperation> = chosen.iter().map(|index| ops[*index].clone()).collect();
            let mut image = base.clone();
            image.apply(&selected);
            images.push((cut, image));
        }
    }
    images
}

fn recover_class(image: &MemoryPool) -> String {
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { .. } => "FileRead".to_string(),
        RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
        RecoveryOutcome::Failed { failure, .. } => {
            let text = format!("{failure:?}");
            format!("Failed:{}", text.chars().take(260).collect::<String>())
        }
    }
}

struct StateResult {
    cut: usize,
    by_record: Vec<&'static str>,
    by_unit: Vec<&'static str>,
    union: u64,
    newest: u64,
    recover: String,
}

fn judge_one(cut: usize, image: &MemoryPool) -> StateResult {
    PROBE_EXEMPT_BY_UNIT.with(|c| c.set(false));
    let by_record = violated_of(image);
    let (union, newest) = counters();
    PROBE_EXEMPT_BY_UNIT.with(|c| c.set(true));
    let by_unit = violated_of(image);
    PROBE_EXEMPT_BY_UNIT.with(|c| c.set(false));
    StateResult { cut, by_record, by_unit, union, newest, recover: recover_class(image) }
}

fn judge_window(pool: &BuiltPool, window: &Window, per_cut_cap: usize) {
    let started = std::time::Instant::now();
    let images = crash_images(pool, window, per_cut_cap);
    let built = started.elapsed();
    let threads = 12usize;
    let results: Vec<StateResult> = std::thread::scope(|scope| {
        let chunk = images.len().div_ceil(threads).max(1);
        let handles: Vec<_> = images
            .chunks(chunk)
            .map(|part| scope.spawn(move || part.iter().map(|(cut, image)| judge_one(*cut, image)).collect::<Vec<_>>()))
            .collect();
        handles.into_iter().flat_map(|h| h.join().expect("线程")).collect()
    });
    let mut violated_histogram: BTreeMap<String, u64> = BTreeMap::new();
    let mut recover_histogram: BTreeMap<String, u64> = BTreeMap::new();
    let (mut union_positive, mut newest_positive, mut differs) = (0u64, 0u64, 0u64);
    let (mut first_union, mut first_newest, mut first_diff) = (None, None, None);
    for r in &results {
        *violated_histogram.entry(format!("{:?}", r.by_record)).or_default() += 1;
        *recover_histogram.entry(r.recover.clone()).or_default() += 1;
        if r.union > 0 { union_positive += 1; first_union.get_or_insert(r.cut); }
        if r.newest > 0 { newest_positive += 1; first_newest.get_or_insert(r.cut); }
        if r.by_record != r.by_unit { differs += 1; first_diff.get_or_insert(r.cut); }
    }
    println!(
        "PROBE-CRASH window={} ops=[{},{}) states={} bit_record_referenced_by_union={union_positive}(first cut {first_union:?}) by_newest_root={newest_positive}(first cut {first_newest:?}) per_unit_vs_per_record_verdict_differs={differs}(first cut {first_diff:?}) build={built:?} total={:?}",
        window.name, window.start, window.end, results.len(), started.elapsed()
    );
    println!("PROBE-CRASH window={} checker_violated_histogram={violated_histogram:?}", window.name);
    println!("PROBE-CRASH window={} recover_histogram={recover_histogram:?}", window.name);
}

fn transient_isolation_with_windows(tag: &str) -> (BuiltPool, SlotNumber, Window) {
    // B 发布的窗口：用同一个构造，但在发布前后各取一次录制流长度
    let probe_pool = build_pool(&format!("{tag}-probe"));
    drop(probe_pool);
    let mut pool = build_pool(tag);
    let before = pool.retained_operations().len();
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let mut wrapped: Vec<(DeviceIdentity, ReadsAtOneOffsetRefusedAFewTimes<Recorded>)> = pool
        .devices
        .take()
        .expect("镜像还开着")
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                ReadsAtOneOffsetRefusedAFewTimes {
                    inner: device,
                    refused_offset: data_slot.to_device_offset(),
                    reads_to_refuse: Cell::new(2),
                },
            )
        })
        .collect();
    let second = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &first,
            FirstFile {
                content: &second_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("B")
    };
    pool.devices = Some(wrapped.into_iter().map(|(identity, device)| (identity, device.inner)).collect());
    pool.output = second;
    let after = pool.retained_operations().len();
    (pool, data_slot, Window { name: "isolating-publish-B", start: before, end: after })
}

#[test]
fn probe_crash_isolating_publish() {
    std::env::remove_var("PROBE_NO_CLEAR");
    let (pool, _slot, window) = transient_isolation_with_windows("probe-crash-b");
    println!("PROBE-CRASH window sizes: B publish ops={}", window.end - window.start);
    judge_window(&pool, &window, 16);
}

fn crash_of_the_rollback(no_clear: bool) {
    if no_clear {
        std::env::set_var("PROBE_NO_CLEAR", "1");
    } else {
        std::env::remove_var("PROBE_NO_CLEAR");
    }
    let (mut pool, _slot, _b) = transient_isolation_with_windows(if no_clear { "probe-crash-rb-noclear" } else { "probe-crash-rb" });
    let start = pool.retained_operations().len();
    roll_back_to_the_first_version(&mut pool).expect("回退照做");
    let end = pool.retained_operations().len();
    let name = if no_clear { "rollback-publish-NO-CLEAR" } else { "rollback-publish" };
    judge_window(&pool, &Window { name, start, end }, 16);
    std::env::remove_var("PROBE_NO_CLEAR");
}

#[test]
fn probe_crash_rollback_publish_clearing() {
    crash_of_the_rollback(false);
}

#[test]
fn probe_crash_rollback_publish_not_clearing() {
    crash_of_the_rollback(true);
}

// ════════════ Q5：写者缺陷让隔离位只落在一块盘上（对称被破坏） ════════════
fn both_readings(label: &str, pool: &BuiltPool) {
    let image = pool.memory_pool();
    PROBE_EXEMPT_BY_UNIT.with(|c| c.set(false));
    let by_record = violated_of(&image);
    PROBE_EXEMPT_BY_UNIT.with(|c| c.set(true));
    let by_unit = violated_of(&image);
    PROBE_EXEMPT_BY_UNIT.with(|c| c.set(false));
    println!(
        "PROBE-Q5 {label}: per_record_violated={by_record:?} per_unit_violated={by_unit:?} recover={}",
        recover_class(&image)
    );
}

fn asymmetric_case(label: &str, device_without_the_bit: DeviceIdentity) {
    std::env::remove_var("PROBE_NO_CLEAR");
    let (mut pool, data_slot) = transient_isolation_pool(label);
    let mut record = *pool.allocator.record_for(device_without_the_bit, data_slot).expect("记录");
    assert!(record.is_quarantined);
    record.is_quarantined = false;
    pool.allocator.probe_replace_record(record);
    println!("PROBE-Q5 {label}: after the writer defect records[{}]", bits_of(&pool, data_slot));
    for (seed, seconds) in [(5, 120), (6, 180), (7, 240), (8, 300)] {
        overwrite(&mut pool, seed, seconds);
    }
    both_readings(&format!("{label} newest txg {} (A still in the ring)", pool.output.root.checkpoint_txg.0), &pool);
    let newest_txg = pool.output.root.checkpoint_txg.0;
    for floor in (4..=newest_txg).rev() {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut current = pool.output.clone();
        let allocator_before = pool.allocator.clone();
        match raise_rollback_floor(&publish_parameters, devices, &mut pool.allocator, &mut current, CheckpointTxg(floor), ShadowLedger::On) {
            Ok(_) => {
                pool.output = current;
                both_readings(&format!("{label} after raising F to {floor}"), &pool);
                break;
            }
            Err(_) => pool.allocator = allocator_before,
        }
    }
}

#[test]
fn probe_asymmetric_bit_on_device_zero_only() {
    asymmetric_case("asym-bit-only-on-d0", DeviceIdentity(1));
}

#[test]
fn probe_asymmetric_bit_on_device_one_only() {
    asymmetric_case("asym-bit-only-on-d1", DeviceIdentity(0));
}

#[test]
fn probe_symmetric_control() {
    std::env::remove_var("PROBE_NO_CLEAR");
    let (mut pool, data_slot) = transient_isolation_pool("sym-control");
    println!("PROBE-Q5 sym-control: records[{}]", bits_of(&pool, data_slot));
    for (seed, seconds) in [(5, 120), (6, 180), (7, 240), (8, 300)] {
        overwrite(&mut pool, seed, seconds);
    }
    let newest_txg = pool.output.root.checkpoint_txg.0;
    for floor in (4..=newest_txg).rev() {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut current = pool.output.clone();
        let allocator_before = pool.allocator.clone();
        match raise_rollback_floor(&publish_parameters, devices, &mut pool.allocator, &mut current, CheckpointTxg(floor), ShadowLedger::On) {
            Ok(_) => {
                pool.output = current;
                both_readings(&format!("sym-control after raising F to {floor}"), &pool);
                break;
            }
            Err(_) => pool.allocator = allocator_before,
        }
    }
}

// ════════════ Q4：回退判「隔离单元」看位还是按推断，写者有缺陷时两种读法的去向 ════════════
fn rollback_result_text(result: &Result<RolledBack, RollbackError>) -> String {
    match result {
        Ok(_) => "Ok(回退做成)".to_string(),
        Err(error) => format!("Err({error:?})"),
    }
}

/// 写者漏释放（B 照常换下 A 的数据单元之后，两块盘的记录被改回仍分配、不带位），再有一份拷贝在盘上坏了；回退到引用它的 A。
fn leaked_unit_with_a_bad_copy(mode: &str) -> String {
    std::env::set_var("PROBE_Q4", mode);
    let mut pool = build_pool(&format!("q4-leak-{mode}"));
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let second = publish_overwrite_in_process(&mut pool, &first, &second_content(), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1)).expect("B");
    pool.output = second;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        pool.allocator.resurrect_released_record(device, singlefs_core::allocator::Placement { slot: data_slot, span: 2 }, CheckpointTxg(3));
    }
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut bytes = read_copy_at(devices, DeviceIdentity(1), data_slot);
    bytes[4096] ^= 0xff;
    write_copy_at(devices, DeviceIdentity(1), data_slot, &bytes);
    let result = roll_back_to_the_first_version(&mut pool);
    let text = rollback_result_text(&result);
    std::env::remove_var("PROBE_Q4");
    text
}

/// 写者的位丢了（隔离时置了位、之后位被清掉：写者缺陷）而单元确实有一份坏，回退到引用它的 A。
fn isolated_unit_whose_bit_got_lost(mode: &str) -> String {
    std::env::set_var("PROBE_Q4", mode);
    let (mut pool, data_slot, _intact) = pool_with_the_data_unit_of_the_first_version_isolated(&format!("q4-lost-bit-{mode}"));
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut record = *pool.allocator.record_for(device, data_slot).expect("记录");
        record.is_quarantined = false;
        pool.allocator.probe_replace_record(record);
    }
    let result = roll_back_to_the_first_version(&mut pool);
    let text = rollback_result_text(&result);
    std::env::remove_var("PROBE_Q4");
    text
}

/// 对照：写者没缺陷的隔离（带位、一份坏），回退到 A。
fn healthy_isolation_with_a_bad_copy(mode: &str) -> String {
    std::env::set_var("PROBE_Q4", mode);
    let (mut pool, _data_slot, _intact) = pool_with_the_data_unit_of_the_first_version_isolated(&format!("q4-healthy-{mode}"));
    let result = roll_back_to_the_first_version(&mut pool);
    let text = rollback_result_text(&result);
    std::env::remove_var("PROBE_Q4");
    text
}

#[test]
fn probe_q4_readings_under_writer_defects() {
    for mode in ["inference", "bit_any", "bit_all"] {
        println!("PROBE-Q4 mode={mode} healthy-isolation+bad-copy: {}", healthy_isolation_with_a_bad_copy(mode));
        println!("PROBE-Q4 mode={mode} leaked-unit(no bit)+bad-copy: {}", leaked_unit_with_a_bad_copy(mode));
        println!("PROBE-Q4 mode={mode} isolation-whose-bit-got-lost+bad-copy: {}", isolated_unit_whose_bit_got_lost(mode));
    }
}

// ════════════ 位跨重挂活下来吗（重建出来的分配器还带不带位） ════════════
#[test]
fn probe_bit_survives_a_writable_remount() {
    std::env::remove_var("PROBE_NO_CLEAR");
    let (mut pool, data_slot) = transient_isolation_pool("bit-remount");
    println!("PROBE-REMOUNT before records[{}]", bits_of(&pool, data_slot));
    let mut devices = pool.reopen_cold();
    let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let text: Vec<String> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| {
            let r = mounted.allocator.record_for(device, data_slot).expect("记录");
            format!("d{}(released={},bit={},gen={})", device.0, r.is_released, r.is_quarantined, r.generation.0)
        })
        .collect();
    println!("PROBE-REMOUNT after cold writable mount records[{}]", text.join(" "));
}
