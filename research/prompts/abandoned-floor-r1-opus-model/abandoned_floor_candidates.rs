//! abandoned-floor-r1 攻方腿（Opus）的副本用例：把第 43 行各候选（环境变量 AF_CANDIDATE）放到复现与相邻历史上跑。
//! 只在攻方副本里；池全在内存稀疏盘上，崩溃状态按录制流前缀取。
mod common;

use common::{memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, raise_rollback_floor_to_the_admission_ceiling,
    roll_back_by_a_forward_publish, unmount, RaiseToTheAdmissionCeiling, RollbackTarget,
    ShadowLedger, Unmounted,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, effective_rollback_floor, instance_table_of_root,
    readable_roots,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream};

type Device = RecordingBlockDevice<SparseBlockDevice>;

fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn devices_of_image(image: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, Device)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect()
}

/// 一个池：内存盘、这个进程的分配器与现行版本、实例号。
struct Pool {
    devices: Vec<(DeviceIdentity, Device)>,
    stream: SharedStream,
    allocator: PoolAllocator,
    output: TransactionOutput,
    instance: InstanceGeneration,
    seed_counter: usize,
}

/// 一次运行里挑出来的一格结局。
#[derive(Clone, Debug)]
struct Verdicts {
    violated: Vec<(String, String)>,
    not_applicable_i31_or_i74: Vec<String>,
}

impl Verdicts {
    fn names(&self) -> String {
        let mut names: Vec<String> = self.violated.iter().map(|(name, detail)| {
            if name == "I-7.4" && detail.contains("被抛弃的根") { "I-7.4(abandoned-half)".to_string() } else { name.clone() }
        }).collect();
        names.sort();
        names.dedup();
        let mut text = format!("[{}]", names.join(","));
        if !self.not_applicable_i31_or_i74.is_empty() {
            text.push_str(&format!(" NA[{}]", self.not_applicable_i31_or_i74.join(",")));
        }
        text
    }
}

fn verdicts_on(image: &MemoryPool) -> Verdicts {
    let mut violated = Vec::new();
    let mut not_applicable = Vec::new();
    for (invariant, verdict) in check_pool_image(image) {
        match verdict {
            InvariantVerdict::Violated(detail) => violated.push((invariant.to_string(), detail)),
            InvariantVerdict::NotApplicable(_) if invariant == "I-3.1" || invariant == "I-7.4" => {
                not_applicable.push(invariant.to_string());
            }
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => {}
        }
    }
    Verdicts { violated, not_applicable_i31_or_i74: not_applicable }
}

/// 环里每条读得出的根：(实例, txg, 带的 F, 按最新根的实例表判被抛弃)。
fn ring_roots(image: &MemoryPool) -> Vec<(u64, u64, u64, bool)> {
    let Ok(system_configuration) = choose_system_configuration(image) else { return Vec::new() };
    let roots = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let table = choose_root(image, &system_configuration).and_then(|newest| instance_table_of_root(image, &newest));
    let mut listed: Vec<(u64, u64, u64, bool)> = roots
        .iter()
        .map(|root| {
            let abandoned = table.as_ref().is_some_and(|table| {
                table.rows.iter().any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
            });
            (u64::from(root.instance.0), root.checkpoint_txg.0, root.rollback_floor.0, abandoned)
        })
        .collect();
    listed.sort_by_key(|(instance, txg, _, _)| (*txg, *instance));
    listed
}

fn implementation_effective_floor(image: &MemoryPool) -> u64 {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    effective_rollback_floor(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .0
}

/// 抬 F 目标 txg 上的根：(有几条, 全属于被抛弃实例)。
fn roots_at_txg(image: &MemoryPool, txg: u64) -> (usize, bool) {
    let at: Vec<_> = ring_roots(image).into_iter().filter(|(_, root_txg, _, _)| *root_txg == txg).collect();
    let all_abandoned = !at.is_empty() && at.iter().all(|(_, _, _, abandoned)| *abandoned);
    (at.len(), all_abandoned)
}

const SYSTEM_CONFIGURATION_REGION_END: u64 = 2 * 4096;

/// 一次操作在录制流里的那一段：从哪一步起、到哪一步止（不含）。
struct OperationSpan {
    image_before: MemoryPool,
    operations: Vec<RetainedOperation>,
}

impl OperationSpan {
    /// 第 n 条根槽 FUA（长 512 的 write_fua）在这一段里的下标。
    fn root_force_unit_access_indexes(&self) -> Vec<usize> {
        self.operations
            .iter()
            .enumerate()
            .filter(|(_, retained)| {
                retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess && retained.operation.length == 512
            })
            .map(|(index, _)| index)
            .collect()
    }
    /// 崩溃状态：这一段里到下标 `last_persisted`（含）为止的操作都持久、之后的都没有。
    fn crash_image_through(&self, last_persisted: usize) -> MemoryPool {
        let mut image = self.image_before.clone();
        image.apply(&self.operations[..=last_persisted]);
        image
    }
    /// 下标区间里的写：(盘, 偏移, 长度)，分系统配置槽与别的。
    fn writes_in(&self, range: std::ops::Range<usize>) -> (Vec<(DeviceIdentity, u64, u64)>, Vec<(DeviceIdentity, u64, u64)>) {
        let mut system_configuration = Vec::new();
        let mut other = Vec::new();
        for retained in &self.operations[range] {
            let operation = &retained.operation;
            if operation.kind == RecordedOperationKind::Barrier {
                continue;
            }
            let entry = (operation.device, operation.offset.0, operation.length);
            if operation.offset.0 < SYSTEM_CONFIGURATION_REGION_END {
                system_configuration.push(entry);
            } else {
                other.push(entry);
            }
        }
        (system_configuration, other)
    }
}

impl Pool {
    fn new() -> Pool {
        let parameters = parameters();
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, Device)> = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        let (instance, output) = {
            let mut writer = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut writer).expect("取号");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            let output = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile { content: &content_seeded_by(3000, 1), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件");
            (instance, output)
        };
        Pool { devices, stream, allocator, output, instance, seed_counter: 0 }
    }

    fn image(&self) -> MemoryPool {
        memory_pool_of_sparse_devices(&self.devices)
    }

    fn fork(&self) -> Pool {
        let stream = SharedStream::retaining_contents();
        Pool {
            devices: devices_of_image(&self.image(), &stream),
            stream,
            allocator: self.allocator.clone(),
            output: self.output.clone(),
            instance: self.instance,
            seed_counter: self.seed_counter,
        }
    }

    fn verdicts(&self) -> Verdicts {
        verdicts_on(&self.image())
    }

    fn overwrite(&mut self) -> Result<TransactionOutput, String> {
        self.seed_counter += 1;
        let seed = 17 + 2 * self.seed_counter;
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let output = publish_overwrite(
            &mut writer,
            &mut self.allocator,
            &self.output,
            FirstFile { content: &content_seeded_by(2500 + seed, seed), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            self.instance,
        )
        .map_err(|error| format!("overwrite:{error:?}"))?;
        self.output = output.clone();
        Ok(output)
    }

    /// 录下一次操作在流里的那一段。
    fn recorded<Result_>(&mut self, operation: impl FnOnce(&mut Pool) -> Result_) -> (Result_, OperationSpan) {
        let image_before = self.image();
        let start = self.stream.operation_count();
        let result = operation(self);
        let operations = self.stream.retained_operations()[start..].to_vec();
        (result, OperationSpan { image_before, operations })
    }

    /// 进程重开：在 `image` 上可写挂载，`hidden` 里的范围在挂载期间读回全 0，挂载交回之后仍是全 0 的写回原样（挂载写过的不动）。
    fn recover_from(image: &MemoryPool, hidden: &[(DeviceIdentity, u64, u64)], seed_counter: usize) -> Result<(Pool, String), String> {
        let stream = SharedStream::retaining_contents();
        let mut devices = devices_of_image(image, &stream);
        let mut saved = Vec::new();
        for (identity, offset, length) in hidden {
            let index = devices.iter().position(|(candidate, _)| candidate == identity).expect("盘");
            let mut bytes = vec![0u8; usize::try_from(*length).expect("长度")];
            devices[index].1.read_at(DeviceOffsetInBytes(*offset), &mut bytes).expect("暂存");
            devices[index].1.write_at(DeviceOffsetInBytes(*offset), &vec![0u8; bytes.len()], WriteDurability::Plain).expect("清零");
            saved.push((index, *offset, bytes));
        }
        let mounted = mount_writable(&parameters(), &mut devices);
        let mut rewritten_by_the_mount = 0;
        for (index, offset, bytes) in &saved {
            let mut now = vec![0u8; bytes.len()];
            devices[*index].1.read_at(DeviceOffsetInBytes(*offset), &mut now).expect("读回");
            if now.iter().all(|byte| *byte == 0) {
                devices[*index].1.write_at(DeviceOffsetInBytes(*offset), bytes, WriteDurability::Plain).expect("写回");
            } else {
                rewritten_by_the_mount += 1;
            }
        }
        let mounted = mounted.map_err(|error| format!("mount:{error:?}"))?;
        let summary = format!(
            "chosen txg {} new instance {} row txg {} isolated {:?} rewritten_hidden {}",
            mounted.output.effective_root.checkpoint_txg.0,
            mounted.output.instance.0,
            mounted.output.row_publish.root().checkpoint_txg.0,
            mounted.output.isolated_slots_per_device.iter().map(|(_, slots)| *slots).collect::<Vec<_>>(),
            rewritten_by_the_mount
        );
        let output = mounted.current.file_version().ok_or("mount:没有带文件的现行版本")?.clone();
        Ok((
            Pool { devices, stream, allocator: mounted.allocator, output, instance: mounted.output.instance, seed_counter },
            summary,
        ))
    }

    fn remount(&mut self) -> Result<String, String> {
        let (pool, summary) = Pool::recover_from(&self.image(), &[], self.seed_counter)?;
        *self = pool;
        Ok(summary)
    }

    fn raise_forced(&mut self, floor: u64) -> Result<String, String> {
        let raised = raise_rollback_floor(
            &parameters(),
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            CheckpointTxg(floor),
            ShadowLedger::On,
        )
        .map_err(|error| format!("raise:{}", short_error(&format!("{error:?}"))))?;
        Ok(format!("ok(F={} reclaimed {} ceiling {})", self.output.root.rollback_floor.0, raised.reclaimed.len(), raised.ceiling.0))
    }

    fn raise_admission(&mut self) -> Result<String, String> {
        let raised = raise_rollback_floor_to_the_admission_ceiling(
            &parameters(),
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            ShadowLedger::On,
        )
        .map_err(|error| format!("admission-raise:{}", short_error(&format!("{error:?}"))))?;
        Ok(match raised {
            RaiseToTheAdmissionCeiling::Raised(raised) => format!("ok(F={} reclaimed {})", self.output.root.rollback_floor.0, raised.reclaimed.len()),
            RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling } => format!("at-ceiling(F={} ceiling {})", floor.0, ceiling.0),
        })
    }

    /// 正常卸载，再可写挂载（新进程）。
    fn unmount_and_remount(&mut self) -> Result<String, String> {
        let mut current = PoolVersion::WithFile(self.output.clone());
        let unmounted = unmount(&parameters(), &mut self.devices, &mut self.allocator, &mut current, ShadowLedger::On)
            .map_err(|error| format!("unmount:{}", short_error(&format!("{error:?}"))))?;
        let floor = match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised.rollback_floor.0,
            Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => 0,
        };
        let summary = self.remount()?;
        Ok(format!("unmount(F={floor}) {summary}"))
    }

    fn roll_back_to(&mut self, instance: u64, txg: u64) -> Result<String, String> {
        roll_back_by_a_forward_publish(
            &parameters(),
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget { instance: InstanceGeneration(u32::try_from(instance).expect("实例号")), checkpoint_txg: CheckpointTxg(txg) },
        )
        .map_err(|error| format!("rollback:{}", short_error(&format!("{error:?}"))))?;
        Ok(format!("rolled back to (inst {instance}, txg {txg})"))
    }
}

fn short_error(text: &str) -> String {
    text.split(['{', '(']).next().unwrap_or(text).trim().to_string()
        + &text.find("requested").map(|at| format!("[{}]", &text[at..text.len().min(at + 90)])).unwrap_or_default()
}

/// 一次发布的根槽与数据单元（「一时读不出」要藏的地方）。
fn root_slot_and_data_units_of(version: &TransactionOutput) -> Vec<(DeviceIdentity, u64, u64)> {
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(version.root.checkpoint_txg, publish_parameters.geometry.root_ring_slots_per_region);
    let root_slot_device = publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(target, publish_parameters.geometry.fixed_structure_slot_spacing);
    let data_unit_bytes = singlefs_format::DATA_UNIT_BYTES;
    std::iter::once((root_slot_device, root_slot_offset.0, u64::from(publish_parameters.geometry.physical_block_size)))
        .chain(
            version
                .data_pointers
                .iter()
                .flat_map(|pointer| pointer.locations)
                .map(|location| (location.device, location.slot.to_device_offset().0, data_unit_bytes)),
        )
        .collect()
}

fn candidate() -> String {
    std::env::var("AF_CANDIDATE").unwrap_or_else(|_| "today".to_string())
}
fn abandoned_floor_mode() -> String {
    std::env::var("AF_ABANDONED_F").unwrap_or_else(|_| "not-counted".to_string())
}
fn cell(experiment: &str, text: &str) {
    eprintln!("AF-CELL cand={} abF={} exp={experiment} {text}", candidate(), abandoned_floor_mode());
}
fn verdict_text(result: &Result<String, String>, pool: &Pool) -> String {
    match result {
        Ok(summary) => format!("{summary}=>{}", pool.verdicts().names()),
        Err(error) => format!("{error}=>{}", pool.verdicts().names()),
    }
}

/// 抬 F 之后由用户决定的几段后缀，各从同一份抬后的盘面分叉。
fn post_actions(after_raise: &Pool) -> String {
    let mut texts = Vec::new();
    {
        let mut pool = after_raise.fork();
        let first = pool.overwrite().map(|_| "ow".to_string());
        let one = verdict_text(&first, &pool);
        let second = pool.overwrite().map(|_| "ow".to_string());
        texts.push(format!("ow2:{one}>{}", verdict_text(&second, &pool)));
    }
    {
        let mut pool = after_raise.fork();
        let remounted = pool.remount();
        let one = verdict_text(&remounted, &pool);
        if remounted.is_ok() {
            let first = pool.overwrite().map(|_| "ow".to_string());
            let two = verdict_text(&first, &pool);
            let second = pool.overwrite().map(|_| "ow".to_string());
            texts.push(format!("remount_ow2:{one}>{two}>{}", verdict_text(&second, &pool)));
        } else {
            texts.push(format!("remount_ow2:{one}"));
        }
    }
    {
        let mut pool = after_raise.fork();
        let unmounted = pool.unmount_and_remount();
        let one = verdict_text(&unmounted, &pool);
        if unmounted.is_ok() {
            let first = pool.overwrite().map(|_| "ow".to_string());
            texts.push(format!("unmount_remount_ow1:{one}>{}", verdict_text(&first, &pool)));
        } else {
            texts.push(format!("unmount_remount_ow1:{one}"));
        }
    }
    {
        // 回退到环里每条没被抛弃、带文件、低于现行那一版的根（带文件与否由回退自己判，拒了照记）。
        let targets: Vec<(u64, u64)> = ring_roots(&after_raise.image())
            .into_iter()
            .filter(|(_, txg, _, abandoned)| !abandoned && *txg >= 3 && *txg < after_raise.output.root.checkpoint_txg.0)
            .map(|(instance, txg, _, _)| (instance, txg))
            .collect();
        for (instance, txg) in targets {
            let mut pool = after_raise.fork();
            let rolled = pool.roll_back_to(instance, txg);
            let one = verdict_text(&rolled, &pool);
            if rolled.is_ok() {
                let first = pool.overwrite().map(|_| "ow".to_string());
                texts.push(format!("rb{txg}_ow1:{one}>{}", verdict_text(&first, &pool)));
            } else {
                texts.push(format!("rb{txg}:{}", rolled.err().unwrap_or_default()));
            }
        }
    }
    texts.join(" | ")
}

fn raise_cell(experiment: &str, keys: &str, base: &Pool, floor: u64, with_post_actions: bool) {
    let mut pool = base.fork();
    let image_before = pool.image();
    let before = verdicts_on(&image_before).names();
    let (roots_at_floor, all_abandoned) = roots_at_txg(&image_before, floor);
    let raised = pool.raise_forced(floor);
    let after = verdict_text(&raised, &pool);
    let posts = if with_post_actions && raised.is_ok() { post_actions(&pool) } else { String::new() };
    cell(
        experiment,
        &format!(
            "{keys} F={floor} txg_before={} roots_at_F={roots_at_floor} all_abandoned={all_abandoned} before={before} raise={after} post=[{posts}]",
            base.output.root.checkpoint_txg.0
        ),
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    /// C 的系统配置轮换没落盘就崩（录制流前缀到 C 的根槽 FUA 为止）；可写挂载时 C 的根槽与数据单元一时读不出。
    CrashBeforeRotation,
    /// C 整次发布落盘；可写挂载时 C 的根槽、数据单元与见证 C 的系统配置槽（每盘一槽）一时读不出。
    DoubleFault,
}

/// A（txg 3）→ B（4）→ C（5）→ 按变体崩溃、可写挂载时 C 一时读不出：落到 B、实例 2。
fn abandon_c(variant: Variant, overwrites_before_b: usize) -> (Pool, String) {
    let mut pool = Pool::new();
    for _ in 0..overwrites_before_b {
        pool.overwrite().expect("预填覆盖写");
    }
    pool.overwrite().expect("B");
    let (third, span) = pool.recorded(|pool| pool.overwrite());
    let third = third.expect("C");
    let root_index = *span.root_force_unit_access_indexes().last().expect("C 的根槽 FUA");
    let (system_configuration_after, other_after) = span.writes_in(root_index + 1..span.operations.len());
    assert!(other_after.is_empty(), "根槽 FUA 之后只有系统配置槽写：{other_after:?}");
    let mut hidden = root_slot_and_data_units_of(&third);
    let image = match variant {
        Variant::CrashBeforeRotation => span.crash_image_through(root_index),
        Variant::DoubleFault => {
            hidden.extend(system_configuration_after.iter().copied());
            span.crash_image_through(span.operations.len() - 1)
        }
    };
    let (recovered, summary) = Pool::recover_from(&image, &hidden, pool.seed_counter).expect("可写挂载");
    (recovered, format!("C=txg {} {summary}", third.root.checkpoint_txg.0))
}

fn run_e1(variant: Variant, remount: bool) {
    let (mut base, summary) = abandon_c(variant, 0);
    let remount_summary = if remount { base.remount().expect("再可写挂载") } else { String::new() };
    cell("E1-setup", &format!("variant={variant:?} remount={remount} {summary} | {remount_summary} ring={:?}", ring_roots(&base.image())));
    for overwrites_after in 0..=6 {
        for floor in 3..=9 {
            raise_cell("E1", &format!("variant={variant:?} remount={remount} k={overwrites_after}"), &base, floor, true);
        }
        base.overwrite().expect("覆盖写");
    }
}

#[test]
fn e1_crash_before_rotation_no_remount() { run_e1(Variant::CrashBeforeRotation, false); }
#[test]
fn e1_crash_before_rotation_remount() { run_e1(Variant::CrashBeforeRotation, true); }
#[test]
fn e1_double_fault_no_remount() { run_e1(Variant::DoubleFault, false); }
#[test]
fn e1_double_fault_remount() { run_e1(Variant::DoubleFault, true); }

/// 同一份盘面上不抬 F、直接走后缀（回退、卸载、覆盖写）：给「抬 F 之前就红没红」「回退得了哪几条」做对照。
fn no_raise_cell(experiment: &str, keys: &str, base: &Pool) {
    let pool = base.fork();
    let before = pool.verdicts().names();
    let posts = post_actions(&pool);
    cell(experiment, &format!("{keys} F=none txg_before={} F_eff_impl={} before={before} post=[{posts}]", base.output.root.checkpoint_txg.0, implementation_effective_floor(&pool.image())));
}

/// A（3）→ B（4）→ C（5）→ D（6）→ E（7）：E 的系统配置轮换没落盘就崩；可写挂载时 D、E 的根槽与数据单元、见证 D 的系统配置槽（每盘一槽）一时读不出。
/// 系统配置里读得出的只剩见证 C 的那一槽：落到 C，D、E 两条根属于被抛弃的实例 1。
fn abandon_d_and_e() -> (Pool, String) {
    let mut pool = Pool::new();
    pool.overwrite().expect("B");
    pool.overwrite().expect("C");
    let (fourth, span_d) = pool.recorded(|pool| pool.overwrite());
    let fourth = fourth.expect("D");
    let (fifth, span_e) = pool.recorded(|pool| pool.overwrite());
    let fifth = fifth.expect("E");
    let d_root = *span_d.root_force_unit_access_indexes().last().expect("D 根槽");
    let (d_rotation, _) = span_d.writes_in(d_root + 1..span_d.operations.len());
    let e_root = *span_e.root_force_unit_access_indexes().last().expect("E 根槽");
    let (_, e_other_after) = span_e.writes_in(e_root + 1..span_e.operations.len());
    assert!(e_other_after.is_empty(), "E 根槽 FUA 之后只有系统配置槽写");
    let mut hidden = root_slot_and_data_units_of(&fourth);
    hidden.extend(root_slot_and_data_units_of(&fifth));
    hidden.extend(d_rotation.iter().copied());
    let image = span_e.crash_image_through(e_root);
    let (recovered, summary) = Pool::recover_from(&image, &hidden, pool.seed_counter).expect("可写挂载");
    (recovered, format!("D=txg {} E=txg {} {summary}", fourth.root.checkpoint_txg.0, fifth.root.checkpoint_txg.0))
}

fn run_e3(remount: bool) {
    let (mut base, summary) = abandon_d_and_e();
    let remount_summary = if remount { base.remount().expect("再可写挂载") } else { String::new() };
    cell("E3-setup", &format!("remount={remount} {summary} | {remount_summary} ring={:?}", ring_roots(&base.image())));
    for overwrites_after in 0..=6 {
        no_raise_cell("E3", &format!("remount={remount} k={overwrites_after}"), &base);
        for floor in 3..=11 {
            raise_cell("E3", &format!("remount={remount} k={overwrites_after}"), &base, floor, true);
        }
        base.overwrite().expect("覆盖写");
    }
}

#[test]
fn e3_two_abandoned_roots_remount() { run_e3(true); }
#[test]
fn e3_two_abandoned_roots_no_remount() { run_e3(false); }

#[derive(Clone, Copy, Debug)]
enum RaiseInTheAbandonedInstance {
    /// 只供测试强制进入的那一档，抬到给定的 F。
    Forced(u64),
    /// 正常卸载那一串（F 抬到现行那一版的 txg）。
    Unmount,
}

/// 实例 1：A（3）→ 三次预填（4、5、6）→ B（7）→ C（8），在 C 上抬 F（强制到给定值，或正常卸载抬到 8）：先写系统配置（SysPre）再发第一次带新 F 的空发布（txg 9）；
/// 崩在那次发布的根槽 FUA 之后、它的轮换之前。可写挂载时那次发布写下的单元、journal 与根槽一时读不出；
/// `hide_the_raised_floor_in_the_system_configuration` 时 SysPre 写的系统配置槽（每盘一槽）也一时读不出——挂载的取号写落回那一槽，新 F 从系统配置里没了，
/// 只剩被抛弃的根 txg 9 带着它。
fn abandon_the_first_raise_publish(mode: RaiseInTheAbandonedInstance, hide_the_raised_floor_in_the_system_configuration: bool) -> Result<(Pool, String), String> {
    let mut pool = Pool::new();
    for _ in 0..3 {
        pool.overwrite().expect("预填");
    }
    pool.overwrite().expect("B");
    pool.overwrite().expect("C");
    let (raised, span) = pool.recorded(|pool| match mode {
        RaiseInTheAbandonedInstance::Forced(floor) => pool.raise_forced(floor),
        RaiseInTheAbandonedInstance::Unmount => {
            let mut current = PoolVersion::WithFile(pool.output.clone());
            unmount(&parameters(), &mut pool.devices, &mut pool.allocator, &mut current, ShadowLedger::On)
                .map(|_| "unmount ok".to_string())
                .map_err(|error| format!("unmount:{}", short_error(&format!("{error:?}"))))
        }
    });
    let raised = raised?;
    let first_root = *span.root_force_unit_access_indexes().first().expect("第一次空发布的根槽");
    if std::env::var_os("AF_STREAM").is_some() {
        for (index, retained) in span.operations.iter().enumerate() {
            eprintln!("AF-STREAM mode={mode:?} #{index}: {}", retained.operation.to_stream_line());
        }
    }
    let (system_configuration_before, other_before) = span.writes_in(0..first_root + 1);
    // journal 记录不藏：藏了它，新实例看不见 txg 9 被用过，写行就落在 txg 9、盖掉那条根槽（试跑看到的，AF-CELL E4-setup rewritten_hidden）。
    let journal_start = singlefs_format::JOURNAL_RING_START_SLOT * singlefs_format::SLOT_BYTES;
    let journal_end = journal_start + parameters().geometry.journal_ring_bytes;
    let journal_writes = other_before.iter().filter(|(_, offset, _)| *offset >= journal_start && *offset < journal_end).count();
    let mut hidden: Vec<(DeviceIdentity, u64, u64)> = other_before
        .into_iter()
        .filter(|(_, offset, _)| !(*offset >= journal_start && *offset < journal_end))
        .collect();
    if std::env::var_os("AF_STREAM").is_some() {
        eprintln!("AF-STREAM mode={mode:?} journal_writes_kept_readable={journal_writes} hidden={hidden:?}");
    }
    if hide_the_raised_floor_in_the_system_configuration {
        hidden.extend(system_configuration_before.iter().copied());
    }
    let image = span.crash_image_through(first_root);
    let (recovered, summary) = Pool::recover_from(&image, &hidden, pool.seed_counter)?;
    Ok((recovered, format!("raise(instance 1)={raised} syspre_writes={} {summary}", system_configuration_before.len())))
}

fn run_e4(mode: RaiseInTheAbandonedInstance, hide: bool) {
    let (mut base, summary) = match abandon_the_first_raise_publish(mode, hide) {
        Ok(built) => built,
        Err(error) => {
            cell("E4-setup", &format!("mode={mode:?} hide_syspre={hide} FAILED {error}"));
            return;
        }
    };
    let remount_summary = base.remount().map_or_else(|error| error, |summary| summary);
    cell("E4-setup", &format!("mode={mode:?} hide_syspre={hide} {summary} | {remount_summary} F_eff_impl={} ring={:?}", implementation_effective_floor(&base.image()), ring_roots(&base.image())));
    for overwrites_after in 0..=4 {
        let keys = format!("mode={mode:?} hide_syspre={hide} k={overwrites_after}");
        no_raise_cell("E4", &keys, &base);
        {
            let mut pool = base.fork();
            let raised = pool.raise_admission();
            let text = verdict_text(&raised, &pool);
            let posts = if raised.as_ref().is_ok_and(|text| text.starts_with("ok")) { post_actions(&pool) } else { String::new() };
            cell("E4", &format!("{keys} F=admission txg_before={} raise={text} post=[{posts}]", base.output.root.checkpoint_txg.0));
        }
        for floor in 3..=12 {
            raise_cell("E4", &keys, &base, floor, true);
        }
        base.overwrite().expect("覆盖写");
    }
}

#[test]
fn e4_forced_four_hidden() { run_e4(RaiseInTheAbandonedInstance::Forced(4), true); }
#[test]
fn e4_forced_five_hidden() { run_e4(RaiseInTheAbandonedInstance::Forced(5), true); }
#[test]
fn e4_unmount_hidden() { run_e4(RaiseInTheAbandonedInstance::Unmount, true); }
#[test]
fn e4_forced_five_not_hidden() { run_e4(RaiseInTheAbandonedInstance::Forced(5), false); }
#[test]
fn e4_unmount_not_hidden() { run_e4(RaiseInTheAbandonedInstance::Unmount, false); }

/// C379 的两个口径：记账行的全空聚簇段数（`empty_segments`，只看分配位）与分配器能开的段（还看隔离位与扣住位）；
/// 「只按分配位算空、却开不出来」的段数与其中整段 64 槽都被挡的段数。
fn segment_accounting(pool: &Pool) -> String {
    let per_segment = 64u64;
    pool.allocator
        .devices
        .iter()
        .map(|device_map| {
            let start = device_map.unit_area_start().slot().0;
            let segments = device_map.unit_area_slots() / per_segment;
            let mut empty_but_blocked = 0u64;
            let mut empty_and_wholly_blocked = 0u64;
            let mut blocked_segment_numbers = Vec::new();
            for segment in 0..segments {
                if !device_map.segment_is_empty(usize::try_from(segment).expect("段号")) {
                    continue;
                }
                let blocked = (0..per_segment)
                    .filter(|offset| !device_map.is_free(singlefs_core::address::SlotNumber(start + segment * per_segment + offset)))
                    .count();
                if blocked > 0 {
                    empty_but_blocked += 1;
                    if blocked == 64 {
                        empty_and_wholly_blocked += 1;
                    }
                    if blocked_segment_numbers.len() < 4 {
                        blocked_segment_numbers.push(format!("{}:{blocked}", start + segment * per_segment));
                    }
                }
            }
            format!(
                "dev{} row_empty_segments={} lowest_openable={:?} empty_but_blocked={empty_but_blocked} wholly_blocked={empty_and_wholly_blocked} isolated={} examples={:?}",
                device_map.device.0,
                device_map.empty_segments(),
                device_map.lowest_empty_segment().map(|slot| slot.0),
                device_map.isolated_slots(),
                blocked_segment_numbers
            )
        })
        .collect::<Vec<_>>()
        .join(" ; ")
}

/// C 写多大的文件、C 之前预填几次：C 那一版独占的单元能不能自然占满一整段（不注入）。
fn abandon_c_with_file_bytes(variant: Variant, prefill: usize, c_file_bytes: usize) -> Result<(Pool, String), String> {
    let mut pool = Pool::new();
    for _ in 0..prefill {
        pool.overwrite()?;
    }
    pool.overwrite()?;
    let parameters = parameters();
    let (third, span) = pool.recorded(|pool| {
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile { content: &content_seeded_by(c_file_bytes, 97), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            pool.instance,
        )
        .map_err(|error| format!("C:{}", short_error(&format!("{error:?}"))))
    });
    let third = third?;
    let root_index = *span.root_force_unit_access_indexes().last().expect("C 的根槽 FUA");
    let (system_configuration_after, other_after) = span.writes_in(root_index + 1..span.operations.len());
    assert!(other_after.is_empty(), "根槽 FUA 之后只有系统配置槽写");
    let mut hidden = root_slot_and_data_units_of(&third);
    let image = match variant {
        Variant::CrashBeforeRotation => span.crash_image_through(root_index),
        Variant::DoubleFault => {
            hidden.extend(system_configuration_after.iter().copied());
            span.crash_image_through(span.operations.len() - 1)
        }
    };
    let data_units = third.data_pointers.len();
    let (recovered, summary) = Pool::recover_from(&image, &hidden, pool.seed_counter)?;
    Ok((recovered, format!("C=txg {} data_units={data_units} {summary}", third.root.checkpoint_txg.0)))
}

fn run_e5(variant: Variant, c_file_bytes: usize) {
    {
        for prefill in 0usize..=45 {
            let keys = format!("variant={variant:?} c_bytes={c_file_bytes} prefill={prefill}");
            match abandon_c_with_file_bytes(variant, prefill, c_file_bytes) {
                Err(error) => cell("E5", &format!("{keys} FAILED {error}")),
                Ok((mut pool, summary)) => {
                    let after_first_mount = segment_accounting(&pool);
                    let first_verdicts = pool.verdicts().names();
                    let remount = pool.remount().map_or_else(|error| error, |summary| summary);
                    let after_remount = segment_accounting(&pool);
                    let remount_verdicts = pool.verdicts().names();
                    let written = pool.overwrite().map(|_| "ow".to_string());
                    let after_write = segment_accounting(&pool);
                    let write_verdicts = verdict_text(&written, &pool);
                    cell("E5", &format!("{keys} {summary} | first_mount: {after_first_mount} v={first_verdicts} | remount {remount}: {after_remount} v={remount_verdicts} | after_ow: {after_write} v={write_verdicts}"));
                }
            }
        }
    }
}

#[test]
fn e5_crash_before_rotation_small_file() { run_e5(Variant::CrashBeforeRotation, 2_500); }
#[test]
fn e5_crash_before_rotation_full_data_unit() { run_e5(Variant::CrashBeforeRotation, 30_000); }
#[test]
fn e5_double_fault_small_file() { run_e5(Variant::DoubleFault, 2_500); }
#[test]
fn e5_double_fault_full_data_unit() { run_e5(Variant::DoubleFault, 30_000); }

/// 产品入口：准入抬 F（抬到上限）与正常卸载，在各条被抛弃实例的历史上 F 落在哪。
fn product_entry_cells(experiment: &str, keys_prefix: &str, base: &Pool) {
    let mut pool = base.fork();
    let image_before = pool.image();
    let raised = pool.raise_admission();
    let landed = pool.output.root.rollback_floor.0;
    let (roots_at_floor, all_abandoned) = roots_at_txg(&image_before, landed);
    let text = verdict_text(&raised, &pool);
    let posts = if raised.as_ref().is_ok_and(|text| text.starts_with("ok")) { post_actions(&pool) } else { String::new() };
    cell(experiment, &format!("{keys_prefix} entry=admission txg_before={} landed_F={landed} roots_at_F={roots_at_floor} all_abandoned={all_abandoned} raise={text} post=[{posts}]", base.output.root.checkpoint_txg.0));
    let mut pool = base.fork();
    let unmounted = pool.unmount_and_remount();
    cell(experiment, &format!("{keys_prefix} entry=unmount txg_before={} result={}", base.output.root.checkpoint_txg.0, verdict_text(&unmounted, &pool)));
}

fn run_e6(label: &str, mut base: Pool) {
    cell("E6-setup", &format!("base={label} ring={:?}", ring_roots(&base.image())));
    for overwrites_after in 0..=12 {
        product_entry_cells("E6", &format!("base={label} k={overwrites_after}"), &base);
        base.overwrite().expect("覆盖写");
    }
}

#[test]
fn e6_product_entries_after_crash_before_rotation_remount() {
    let (mut base, _) = abandon_c(Variant::CrashBeforeRotation, 0);
    base.remount().expect("再挂载");
    run_e6("E1-CBR-remount", base);
}
#[test]
fn e6_product_entries_after_crash_before_rotation_no_remount() {
    let (base, _) = abandon_c(Variant::CrashBeforeRotation, 0);
    run_e6("E1-CBR-noremount", base);
}
#[test]
fn e6_product_entries_after_two_abandoned_roots() {
    let (mut base, _) = abandon_d_and_e();
    base.remount().expect("再挂载");
    run_e6("E3-remount", base);
}
#[test]
fn e6_product_entries_after_abandoned_raise() {
    let (mut base, _) = abandon_the_first_raise_publish(RaiseInTheAbandonedInstance::Unmount, true).expect("E4 起点");
    base.remount().expect("再挂载");
    run_e6("E4-unmount-hidden-remount", base);
}

#[derive(Clone, Copy, Debug)]
enum UnreadableC {
    /// 只有 C 的根槽一直读不出。
    RootSlot,
    /// C 的根槽与 C 那次发布的 journal 记录一直读不出。
    RootSlotAndJournalRecord,
    /// C 的根槽与 C 的数据单元（两份）一直读不出（记录施加前验点名单元失败）。
    RootSlotAndDataUnits,
}

/// 不被抛弃、只是读不出的根也可能留下空档：A（3）→ B（4）→ C（5）→ D（6），C 的某几处之后一直读不出（清零、不写回），再可写挂载（落到 D）。
fn permanently_unreadable_c(what: UnreadableC) -> (Pool, String) {
    let mut pool = Pool::new();
    pool.overwrite().expect("B");
    let (third, span) = pool.recorded(|pool| pool.overwrite());
    let third = third.expect("C");
    pool.overwrite().expect("D");
    let journal_start = singlefs_format::JOURNAL_RING_START_SLOT * singlefs_format::SLOT_BYTES;
    let journal_end = journal_start + parameters().geometry.journal_ring_bytes;
    let (_, other) = span.writes_in(0..span.operations.len());
    let root_and_data = root_slot_and_data_units_of(&third);
    let zeroed: Vec<(DeviceIdentity, u64, u64)> = match what {
        UnreadableC::RootSlot => vec![root_and_data[0]],
        UnreadableC::RootSlotAndJournalRecord => std::iter::once(root_and_data[0])
            .chain(other.iter().copied().filter(|(_, offset, _)| *offset >= journal_start && *offset < journal_end))
            .collect(),
        UnreadableC::RootSlotAndDataUnits => root_and_data,
    };
    for (identity, offset, length) in &zeroed {
        let index = pool.devices.iter().position(|(candidate, _)| candidate == identity).expect("盘");
        pool.devices[index].1.write_at(DeviceOffsetInBytes(*offset), &vec![0u8; usize::try_from(*length).expect("长度")], WriteDurability::Plain).expect("清零");
    }
    let summary = pool.remount().expect("可写挂载");
    (pool, format!("zeroed={} {summary}", zeroed.len()))
}

fn run_e7(what: UnreadableC) {
    let (mut base, summary) = permanently_unreadable_c(what);
    cell("E7-setup", &format!("what={what:?} {summary} ring={:?}", ring_roots(&base.image())));
    for overwrites_after in 0..=6 {
        let keys = format!("what={what:?} k={overwrites_after}");
        no_raise_cell("E7", &keys, &base);
        for floor in 3..=9 {
            raise_cell("E7", &keys, &base, floor, true);
        }
        base.overwrite().expect("覆盖写");
    }
}

#[test]
fn e7_unreadable_root_slot() { run_e7(UnreadableC::RootSlot); }
#[test]
fn e7_unreadable_root_slot_and_journal_record() { run_e7(UnreadableC::RootSlotAndJournalRecord); }
#[test]
fn e7_unreadable_root_slot_and_data_units() { run_e7(UnreadableC::RootSlotAndDataUnits); }

/// 根环转圈：B（txg 4）与 C（txg 5）离开根环前后，F 抬到 5 或 6。
#[test]
fn e8_ring_rotation() {
    let (mut base, _) = abandon_c(Variant::CrashBeforeRotation, 0);
    base.remount().expect("再挂载");
    for overwrites_after in 0..=34 {
        for floor in [5u64, 6] {
            let keys = format!("k={overwrites_after}");
            let mut pool = base.fork();
            let image_before = pool.image();
            let before = verdicts_on(&image_before).names();
            let (roots_at_floor, all_abandoned) = roots_at_txg(&image_before, floor);
            let raised = pool.raise_forced(floor);
            let after = verdict_text(&raised, &pool);
            let mut posts = String::new();
            if raised.is_ok() {
                let mut remounted = pool.fork();
                let one = remounted.remount().map_or_else(|error| error, |summary| summary);
                let one_verdicts = remounted.verdicts().names();
                let written = remounted.overwrite().map(|_| "ow".to_string());
                posts = format!("remount_ow1:{one}=>{one_verdicts}>{}", verdict_text(&written, &remounted));
            }
            cell("E8", &format!("{keys} F={floor} txg_before={} roots_at_F={roots_at_floor} all_abandoned={all_abandoned} ring_min_txg={:?} before={before} raise={after} post=[{posts}]", base.output.root.checkpoint_txg.0, ring_roots(&image_before).first().map(|root| root.1)));
        }
        base.overwrite().expect("覆盖写");
    }
}

/// 空档里一条根都没有：A（3）→ B（4）→ C（5）崩在 C 的根槽 FUA 之前（录制流前缀到根槽 FUA 的前一步：C 的单元与 journal 记录都落了盘），
/// 可写挂载时 C 的数据单元一时读不出（记录施加前验点名单元失败、不施加）：落到 B，txg 5 只有一条不施加的记录、没有根槽。
fn c_without_a_root_slot(hide_data_units: bool) -> (Pool, String) {
    let mut pool = Pool::new();
    pool.overwrite().expect("B");
    let (third, span) = pool.recorded(|pool| pool.overwrite());
    let third = third.expect("C");
    let root_index = *span.root_force_unit_access_indexes().last().expect("C 的根槽 FUA");
    let image = span.crash_image_through(root_index - 1);
    let hidden: Vec<(DeviceIdentity, u64, u64)> = if hide_data_units { root_slot_and_data_units_of(&third).into_iter().skip(1).collect() } else { Vec::new() };
    let (recovered, summary) = Pool::recover_from(&image, &hidden, pool.seed_counter).expect("可写挂载");
    (recovered, format!("C=txg {} (no root slot, data units hidden {hide_data_units}) {summary}", third.root.checkpoint_txg.0))
}

fn run_e9(remount: bool, hide_data_units: bool) {
    let (mut base, summary) = c_without_a_root_slot(hide_data_units);
    let remount_summary = if remount { base.remount().expect("再可写挂载") } else { String::new() };
    cell("E9-setup", &format!("remount={remount} hide_data={hide_data_units} {summary} | {remount_summary} ring={:?}", ring_roots(&base.image())));
    for overwrites_after in 0..=6 {
        let keys = format!("remount={remount} hide_data={hide_data_units} k={overwrites_after}");
        no_raise_cell("E9", &keys, &base);
        for floor in 3..=9 {
            raise_cell("E9", &keys, &base, floor, true);
        }
        {
            let mut pool = base.fork();
            let image_before = pool.image();
            let raised = pool.raise_admission();
            let landed = pool.output.root.rollback_floor.0;
            let (roots_at_floor, all_abandoned) = roots_at_txg(&image_before, landed);
            cell("E9", &format!("{keys} F=admission landed_F={landed} roots_at_F={roots_at_floor} all_abandoned={all_abandoned} raise={}", verdict_text(&raised, &pool)));
        }
        base.overwrite().expect("覆盖写");
    }
}

#[test]
fn e9_gap_without_a_root_slot_remount() { run_e9(true, true); }
#[test]
fn e9_gap_without_a_root_slot_no_remount() { run_e9(false, true); }
#[test]
fn e9_version_applied_only_by_its_record_remount() { run_e9(true, false); }
#[test]
fn e9_version_applied_only_by_its_record_no_remount() { run_e9(false, false); }

/// 候选根一时读不出时抬 F：E1 的起点（崩在 C 的轮换之前、落到 B、实例 2 写行 6 暖机 7）之后进程重开，
/// 这一次可写挂载与之后的覆盖写、抬 F 期间，txg `hidden_txg`（实例 2 的一条有效根，F 之上）的根槽读回全 0；抬完之后读得出了。
fn run_e10(hidden_txg: u64) {
    let (base, summary) = abandon_c(Variant::CrashBeforeRotation, 0);
    cell("E10-setup", &format!("hidden_txg={hidden_txg} {summary} ring={:?}", ring_roots(&base.image())));
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(CheckpointTxg(hidden_txg), publish_parameters.geometry.root_ring_slots_per_region);
    let device = publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = singlefs_core::root_ring::slot_offset(target, publish_parameters.geometry.fixed_structure_slot_spacing).0;
    let length = u64::from(publish_parameters.geometry.physical_block_size);
    for overwrites_after in 3..=6 {
        for floor in [5u64, 6, 7] {
            let keys = format!("hidden_txg={hidden_txg} k={overwrites_after} F={floor}");
            let stream = SharedStream::retaining_contents();
            let mut devices = devices_of_image(&base.image(), &stream);
            let index = devices.iter().position(|(candidate, _)| *candidate == device).expect("盘");
            let mut saved = vec![0u8; usize::try_from(length).expect("长度")];
            devices[index].1.read_at(DeviceOffsetInBytes(offset), &mut saved).expect("暂存");
            devices[index].1.write_at(DeviceOffsetInBytes(offset), &vec![0u8; saved.len()], WriteDurability::Plain).expect("清零");
            let mounted = match mount_writable(&parameters(), &mut devices) {
                Ok(mounted) => mounted,
                Err(error) => {
                    cell("E10", &format!("{keys} mount:{}", short_error(&format!("{error:?}"))));
                    continue;
                }
            };
            let output = mounted.current.file_version().expect("带文件").clone();
            let mut pool = Pool { devices, stream, allocator: mounted.allocator, output, instance: mounted.output.instance, seed_counter: base.seed_counter };
            let mut steps = Vec::new();
            for _ in 0..overwrites_after {
                if let Err(error) = pool.overwrite() {
                    steps.push(error);
                }
            }
            let raised = pool.raise_forced(floor);
            let raise_text = match &raised { Ok(text) => text.clone(), Err(error) => error.clone() };
            let after_raise_hidden = pool.verdicts().names();
            let mut suffix = Vec::new();
            for _ in 0..3 {
                let written = pool.overwrite().map(|_| "ow".to_string());
                suffix.push(match written { Ok(_) => "ow".to_string(), Err(error) => error });
            }
            let hidden_verdicts = pool.verdicts().names();
            let mut now = vec![0u8; saved.len()];
            pool.devices[index].1.read_at(DeviceOffsetInBytes(offset), &mut now).expect("读回");
            let restored = now.iter().all(|byte| *byte == 0);
            if restored {
                pool.devices[index].1.write_at(DeviceOffsetInBytes(offset), &saved, WriteDurability::Plain).expect("写回");
            }
            let readable_again = pool.verdicts().names();
            let remounted = pool.remount();
            let after_remount = verdict_text(&remounted, &pool);
            cell("E10", &format!("{keys} instance={} steps={steps:?} raise={raise_text} after_raise_while_hidden={after_raise_hidden} after_3ow_while_hidden={hidden_verdicts} suffix={suffix:?} restored={restored} readable_again={readable_again} remount={after_remount}", pool.instance.0));
        }
    }
}

#[test]
fn e10_candidate_root_six_unreadable_during_the_raise() { run_e10(6); }
#[test]
fn e10_candidate_root_seven_unreadable_during_the_raise() { run_e10(7); }

/// C379 自然历史第二形：被抛弃根的单元只隔离了段里的一部分，同段里别的单元之后被释放、回收——段在分配位上全空（记账行数它），
/// 分配器因为隔离位不开它。起点：C 被抛弃、再可写挂载（影子账隔离 C 的单元）；之后覆盖写 k 次，产品入口抬 F（准入抬到上限），再覆盖写 j 次。
fn run_e11(variant: Variant, remount_first: bool) {
    let (mut base, summary) = abandon_c(variant, 0);
    if remount_first {
        base.remount().expect("再挂载");
    }
    cell("E11-setup", &format!("variant={variant:?} remount={remount_first} {summary} | {}", segment_accounting(&base)));
    for overwrites_after in 0..=14 {
        let mut pool = base.fork();
        let raised = pool.raise_admission();
        let after_raise = segment_accounting(&pool);
        let after_raise_verdicts = pool.verdicts().names();
        let mut after_writes = Vec::new();
        for _ in 0..3 {
            let written = pool.overwrite();
            after_writes.push(format!("{}:{}", if written.is_ok() { "ow" } else { "ow-err" }, segment_accounting(&pool)));
        }
        let remounted = pool.remount().map_or_else(|error| error, |summary| summary);
        cell(
            "E11",
            &format!(
                "variant={variant:?} remount={remount_first} k={overwrites_after} raise={} v={after_raise_verdicts} | after_raise: {after_raise} | {} | remount {remounted}: {} v={}",
                raised.map_or_else(|error| error, |text| text),
                after_writes.join(" | "),
                segment_accounting(&pool),
                pool.verdicts().names()
            ),
        );
        base.overwrite().expect("覆盖写");
    }
}

#[test]
fn e11_c379_partial_isolation_crash_before_rotation_remount() { run_e11(Variant::CrashBeforeRotation, true); }
#[test]
fn e11_c379_partial_isolation_double_fault_remount() { run_e11(Variant::DoubleFault, true); }
#[test]
fn e11_c379_partial_isolation_crash_before_rotation_no_remount() { run_e11(Variant::CrashBeforeRotation, false); }
#[test]
fn e11_c379_partial_isolation_two_abandoned_roots() {
    let (mut base, summary) = abandon_d_and_e();
    base.remount().expect("再挂载");
    cell("E11-setup", &format!("two_abandoned {summary} | {}", segment_accounting(&base)));
    for overwrites_after in 0..=14 {
        let mut pool = base.fork();
        let raised = pool.raise_admission();
        let after_raise = segment_accounting(&pool);
        let mut after_writes = Vec::new();
        for _ in 0..3 {
            let written = pool.overwrite();
            after_writes.push(format!("{}:{}", if written.is_ok() { "ow" } else { "ow-err" }, segment_accounting(&pool)));
        }
        let remounted = pool.remount().map_or_else(|error| error, |summary| summary);
        cell("E11", &format!("two_abandoned k={overwrites_after} raise={} | after_raise: {after_raise} | {} | remount {remounted}: {} v={}", raised.map_or_else(|error| error, |text| text), after_writes.join(" | "), segment_accounting(&pool), pool.verdicts().names()));
        base.overwrite().expect("覆盖写");
    }
}
