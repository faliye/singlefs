//! m4-publish-lookup-code-r1 攻方腿的差分装置：同一份源码拷进改前（补丁反向）与改后两棵树的 `crates/singlefs-harness/tests/`，
//! 两份二进制跑同一组历史，逐步打出录制流（写、FUA、清零、discard、屏障，带内容哈希）的累计摘要、结局与内存里版本的摘要。
//! 两边逐行相同 ⇒ 写流逐字节相同 ⇒ 每个崩溃状态（写流前缀加段内子集）也相同。只用改前改后都有的 API。
mod common;

use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{DeviceIdentity, InodeNumber, TreeIdentifier};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{CompactionBatchSize, CompactionBatchUnits, CompactionSwitches};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_compaction_switches, raise_rollback_floor_to_the_admission_ceiling,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::fault_injection::SharedFaultPlan;
use singlefs_harness::history::{
    execute_history_handing_out_every_step_image, generate_history_with_weights,
    GenerationWeights, HistoryExecution, HistorySeed,
};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Disk = RecordingBlockDevice<SparseBlockDevice>;
type Disks = Vec<(DeviceIdentity, Disk)>;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha(bytes: &[u8]) -> String {
    hex(&singlefs_harness::sha256::sha256_digest(bytes))
}

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name).map_or(default, |value| value.parse().expect("十进制整数"))
}

/// 录制流从 `from` 起的新操作接到上一段摘要后面再取摘要（累计摘要：任一步写流不同，之后每一行都不同）。
fn chained_stream_digest(previous: &str, stream: &SharedStream, from: usize) -> (String, usize) {
    let operations = stream.operations();
    let mut text = previous.to_owned();
    for operation in &operations[from..] {
        text.push('\n');
        text.push_str(&operation.to_stream_line());
    }
    (sha(text.as_bytes()), operations.len())
}

/// 一版的摘要：单元字节只进 FNV 哈希（Debug 把字节逐个打成十进制，整版打出来太慢），别的字段照 Debug 原样进。
fn version_digest(version: &TransactionOutput) -> String {
    let mut stripped = version.clone();
    let mut text = String::new();
    for unit in &mut stripped.units {
        text.push_str(&format!("{:?}@{}:{:016x};", unit.identity, unit.slot.0, singlefs_harness::fnv1a_64(&unit.bytes)));
        unit.bytes = Vec::new();
    }
    text.push_str(&format!("{stripped:?}"));
    sha(text.as_bytes())
}

fn compaction_run_text(run: &singlefs_core::mounted_session::CompactionRun) -> String {
    let versions: Vec<String> = run.versions_published.iter().map(version_digest).collect();
    format!("versions={versions:?} batches={} outcome={:?} discards={:?}", run.batches_published, run.outcome, run.discards)
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "（非字符串 panic）".to_owned())
}

/// 随机历史：每组具名比重、每个种子各一段，交出每一步的镜像摘要、每一步结局的摘要、结局与整条写流的摘要。
#[test]
fn random_histories_print_digests() {
    let seed_start = environment_number("M4A_SEED_START", 0);
    let seeds = environment_number("M4A_SEEDS", 2);
    let operations = usize::try_from(environment_number("M4A_OPERATIONS", 30)).expect("步数");
    let only_weights = std::env::var("M4A_WEIGHTS").ok();
    for (weights_index, weights) in GenerationWeights::NAMED.iter().enumerate() {
        if let Some(only) = &only_weights {
            if !only.split(',').any(|index| index == weights_index.to_string()) {
                continue;
            }
        }
        for seed in seed_start..seed_start + seeds {
            let history = generate_history_with_weights(HistorySeed(seed), operations, weights);
            let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
            let stream = SharedStream::new();
            let mut step_lines: Vec<String> = Vec::new();
            let mut outcome_lines: Vec<String> = Vec::new();
            let run = execute_history_handing_out_every_step_image(
                &history,
                execution,
                &stream,
                &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
                &mut |observation| {
                    outcome_lines.push(format!(
                        "{:?} {}",
                        observation.position,
                        sha(format!("{:?}", observation.outcome).as_bytes())
                    ));
                },
                &mut |step_image| {
                    let mut written = Vec::new();
                    for (identity, device) in &step_image.image.devices {
                        written.extend_from_slice(&identity.0.to_le_bytes());
                        for (sector, bytes) in device.written_sectors() {
                            written.extend_from_slice(&sector.to_le_bytes());
                            written.extend_from_slice(bytes);
                        }
                    }
                    step_lines.push(format!(
                        "{:?} {:?} operations={} image={}",
                        step_image.position,
                        step_image.operation_kind,
                        step_image.operations_recorded,
                        sha(&written)
                    ));
                },
            );
            let (stream_digest, stream_operations) = chained_stream_digest("", &stream, 0);
            let ending = format!("{:?}", run.ending);
            let ending_name = ending.split([' ', '{', '(']).next().unwrap_or("").to_owned();
            println!(
                "M4A history weights={weights_index} seed={seed} operations={operations} ending={ending_name} ending_digest={} outcomes={} steps_imaged={} stream_operations={stream_operations} stream={stream_digest}",
                sha(ending.as_bytes()),
                run.outcomes.len(),
                step_lines.len()
            );
            for line in step_lines {
                println!("M4A step weights={weights_index} seed={seed} {line}");
            }
            for line in outcome_lines {
                println!("M4A outcome weights={weights_index} seed={seed} {line}");
            }
        }
    }
}

/// 两块录制的 4 GiB 稀疏盘上 mkfs → 取号 → 暖机 → 新池新建文件，再按 `switches` 可写挂载（同七·6 用例的 `formatted_and_mounted`，盘换成录制的）。
fn formatted_and_mounted(stream: &SharedStream, switches: CompactionSwitches) -> (Disks, MountedSession) {
    let parameters = common::parameters();
    let mut devices: Disks = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let disk = SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            (identity, RecordingBlockDevice::with_shared_stream(identity, disk, stream.clone()))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file = common::file_content();
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile { content: &first_file, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable_with_compaction_switches(&parameters, &mut devices, switches).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (devices, session)
}

fn switches_with_batch(batch_units: usize) -> CompactionSwitches {
    let mut switches = CompactionSwitches::PRODUCT;
    switches.batch_size = CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(batch_units).expect("≥1"));
    switches
}

fn content_of_data_units(data_units: usize, seed: u64, short_by: usize) -> Vec<u8> {
    let mut state = seed | 1;
    (0..(data_units * data_unit_payload_capacity()).saturating_sub(short_by))
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// 一段写死前缀、按种子放开用户动作的历史里的一步。
#[derive(Clone, Copy, Debug)]
enum Step {
    NewInodes(u64),
    WriteInode { which: usize, units: usize, fill: u64 },
    OverwriteInode { which: usize, units: usize, fill: u64 },
    OverwriteFirstFile { units: usize, fill: u64 },
    AppendFirstFile { units: usize, fill: u64 },
    TruncateFirstFile { keep_bytes: u64 },
    Unlink { which: usize },
    CreateSnapshot,
    DestroyOldestSnapshot,
    RaiseFloor,
    Compact { batch_units: usize, runs: u64 },
}

struct Scripted {
    stream: SharedStream,
    devices: Disks,
    session: MountedSession,
    inodes: Vec<InodeNumber>,
    time: u64,
    digest: String,
    seen_operations: usize,
}

impl Scripted {
    fn version(&self) -> &TransactionOutput {
        self.session.current.version_of_trees().expect("有树")
    }

    fn snapshots(&self) -> Vec<TreeIdentifier> {
        self.version()
            .tree_table_entries
            .iter()
            .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
            .map(|entry| entry.tree)
            .collect()
    }

    /// 走一步，交回这一步结局的文字（Ok / Err 带 Debug 的摘要）。
    fn step(&mut self, step: Step) -> String {
        self.time += 1;
        let time = self.time;
        let change_result = |session: &mut MountedSession, devices: &mut Disks, change: UserChange<'_>| {
            match session.publish_user_change(devices, change) {
                Ok(published) => {
                    let versions: Vec<String> = published.versions_published.iter().map(version_digest).collect();
                    let text = format!(
                        "floor_raises={:?} refusals={:?} versions={versions:?} compaction={:?} discards={:?}",
                        published.floor_raises,
                        published.refusals_that_pushed_the_floor_raises,
                        published.compaction_after_the_change.as_ref().map(compaction_run_text),
                        published.discards_after_the_change
                    );
                    format!("ok {}", sha(text.as_bytes()))
                }
                Err(refusal) => format!("refused {}", sha(format!("{refusal:?}").as_bytes())),
            }
        };
        match step {
            Step::NewInodes(count) => {
                let first = self.version().inode_number_watermark();
                let text = change_result(&mut self.session, &mut self.devices, UserChange::NewInodes { count, write_time_seconds: time });
                if text.starts_with("ok") {
                    self.inodes.extend((first..first + count).map(InodeNumber));
                }
                text
            }
            Step::WriteInode { which, units, fill } | Step::OverwriteInode { which, units, fill } => {
                let Some(inode) = self.inodes.get(which % self.inodes.len().max(1)).copied() else {
                    return "no-inode".to_owned();
                };
                // 覆盖写只认一个数据单元以内的内容（`UserChange::OverwriteOfAnInode`）：长度按 `units` 在一个单元里变。
                let content = if matches!(step, Step::WriteInode { .. }) {
                    content_of_data_units(units, fill, 5)
                } else {
                    content_of_data_units(1, fill, 5 + 97 * units)
                };
                let change = if matches!(step, Step::WriteInode { .. }) {
                    UserChange::SequentialWriteOfAnInode { inode, content: &content, write_time_seconds: time }
                } else {
                    UserChange::OverwriteOfAnInode { inode, content: &content, write_time_seconds: time }
                };
                change_result(&mut self.session, &mut self.devices, change)
            }
            Step::OverwriteFirstFile { units, fill } | Step::AppendFirstFile { units, fill } => {
                let content = if matches!(step, Step::OverwriteFirstFile { .. }) {
                    content_of_data_units(1, fill, 7 + 97 * units)
                } else {
                    content_of_data_units(units, fill, 7)
                };
                let file = FirstFile { content: &content, write_time_seconds: time };
                let change = if matches!(step, Step::OverwriteFirstFile { .. }) {
                    UserChange::Overwrite(file)
                } else {
                    UserChange::SequentialWrite(file)
                };
                change_result(&mut self.session, &mut self.devices, change)
            }
            Step::TruncateFirstFile { keep_bytes } => change_result(
                &mut self.session,
                &mut self.devices,
                UserChange::Truncate { inode: InodeNumber(singlefs_core::transaction::FIRST_INODE_NUMBER), new_size: keep_bytes, write_time_seconds: time },
            ),
            Step::Unlink { which } => {
                if self.inodes.is_empty() {
                    return "no-inode".to_owned();
                }
                let inode = self.inodes.remove(which % self.inodes.len());
                change_result(&mut self.session, &mut self.devices, UserChange::Unlink { inode, write_time_seconds: time })
            }
            Step::CreateSnapshot => change_result(&mut self.session, &mut self.devices, UserChange::CreateSnapshot),
            Step::DestroyOldestSnapshot => match self.snapshots().first().copied() {
                Some(snapshot) => change_result(&mut self.session, &mut self.devices, UserChange::DestroySnapshot { snapshot }),
                None => "no-snapshot".to_owned(),
            },
            Step::RaiseFloor => {
                let parameters = common::parameters();
                let mut current = self.version().clone();
                let result = raise_rollback_floor_to_the_admission_ceiling(
                    &parameters,
                    &mut self.devices,
                    &mut self.session.allocator,
                    &mut current,
                    self.session.shadow_ledger,
                );
                let tag = if result.is_ok() { "ok" } else { "refused" };
                let text = format!("{tag} {} version={}", sha(format!("{result:?}").as_bytes()), version_digest(&current));
                if result.is_ok() {
                    self.session.current = PoolVersion::of_a_version_of_trees(current);
                }
                text
            }
            Step::Compact { batch_units, runs } => {
                self.session.allocator.set_compaction_switches(switches_with_batch(batch_units));
                let mut texts = Vec::new();
                for _ in 0..runs {
                    let run = self.session.run_compaction(&mut self.devices);
                    let published = run.as_ref().map_or(0, |run| run.versions_published.len());
                    let text = match &run {
                        Ok(run) => compaction_run_text(run),
                        Err(refused) => format!("refused {refused:?}"),
                    };
                    texts.push(format!("{}:{published}", sha(text.as_bytes())));
                    if published == 0 {
                        break;
                    }
                }
                texts.join(",")
            }
        }
    }
}

/// xorshift64：后缀里用户动作的取样。
fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn drawn_step(state: &mut u64, files: u64, units_per_file: usize) -> Step {
    let which = usize::try_from(next(state) % files.max(1)).expect("下标");
    let units = 1 + usize::try_from(next(state) % u64::try_from(units_per_file * 2).expect("单元")).expect("单元");
    let fill = next(state);
    match next(state) % 14 {
        0 => Step::NewInodes(1 + next(state) % 4),
        1 | 2 => Step::WriteInode { which, units, fill },
        3 | 4 => Step::OverwriteInode { which, units, fill },
        5 => Step::OverwriteFirstFile { units, fill },
        6 => Step::AppendFirstFile { units: 1 + units / 2, fill },
        7 => Step::TruncateFirstFile { keep_bytes: next(state) % 70_000 },
        8 => Step::Unlink { which },
        9 => Step::CreateSnapshot,
        10 => Step::DestroyOldestSnapshot,
        11 => Step::RaiseFloor,
        _ => Step::Compact { batch_units: 1 + usize::try_from(next(state) % 8).expect("批"), runs: 1 + next(state) % 6 },
    }
}

fn run_scripted_history(files: u64, units_per_file: usize, seed: u64, drawn_steps: u64) {
    let stream = SharedStream::new();
    let (devices, session) = formatted_and_mounted(&stream, CompactionSwitches::PRODUCT);
    let mut scripted = Scripted {
        stream,
        devices,
        session,
        inodes: Vec::new(),
        time: common::FIXED_WRITE_TIME_SECONDS + 600,
        digest: String::new(),
        seen_operations: 0,
    };
    // 写死的前缀：建文件、整份写、中途建快照、整份重写下标不是 4 的倍数的那几个、抬 F、整理到停（七·6 用例的配方加一个快照）。
    let mut steps: Vec<Step> = vec![Step::NewInodes(files)];
    for which in 0..files {
        let which = usize::try_from(which).expect("下标");
        steps.push(Step::WriteInode { which, units: units_per_file, fill: 1000 + u64::try_from(which).expect("号") });
        if which == usize::try_from(files / 2).expect("半") {
            steps.push(Step::CreateSnapshot);
        }
    }
    for which in 0..files {
        if which % 4 != 0 {
            steps.push(Step::WriteInode { which: usize::try_from(which).expect("下标"), units: units_per_file, fill: 2000 + which });
        }
    }
    for _ in 0..4 {
        steps.push(Step::RaiseFloor);
        steps.push(Step::NewInodes(1));
    }
    steps.push(Step::RaiseFloor);
    steps.push(Step::Compact { batch_units: 2, runs: 64 });
    let prefix = steps.len();
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    for _ in 0..drawn_steps {
        steps.push(drawn_step(&mut state, files, units_per_file));
    }
    for (index, step) in steps.iter().enumerate() {
        let outcome = catch_unwind(AssertUnwindSafe(|| scripted.step(*step)));
        let (digest, seen) = chained_stream_digest(&scripted.digest, &scripted.stream, scripted.seen_operations);
        scripted.digest = digest;
        scripted.seen_operations = seen;
        let version_digest = scripted
            .session
            .current
            .version_of_trees()
            .map_or_else(|| "none".to_owned(), version_digest);
        let mapping_nodes = scripted
            .session
            .current
            .version_of_trees()
            .map_or(0, |version| version.central_mapping_tree.shape.nodes().len());
        let part = if index < prefix { "prefix" } else { "drawn" };
        match outcome {
            Ok(text) => println!(
                "M4B shape={files}x{units_per_file} seed={seed} step={index} {part} {step:?} => {text} operations={seen} stream={} version={version_digest} mapping_nodes={mapping_nodes}",
                scripted.digest
            ),
            Err(payload) => {
                println!(
                    "M4B shape={files}x{units_per_file} seed={seed} step={index} {part} {step:?} => PANIC {} operations={seen} stream={}",
                    panic_text(payload.as_ref()),
                    scripted.digest
                );
                return;
            }
        }
    }
}

/// 写死前缀加按种子放开的用户动作：形状 `M4B_SHAPES`（如 `16x4,48x8`），种子 `M4B_SEED_START` 起 `M4B_SEEDS` 个，后缀 `M4B_STEPS` 步。
#[test]
fn scripted_histories_print_digests() {
    let shapes = std::env::var("M4B_SHAPES").unwrap_or_else(|_| "16x4".to_owned());
    let seed_start = environment_number("M4B_SEED_START", 1);
    let seeds = environment_number("M4B_SEEDS", 1);
    let drawn_steps = environment_number("M4B_STEPS", 20);
    for shape in shapes.split(',') {
        let (files, units) = shape.split_once('x').expect("形如 16x4");
        let files: u64 = files.parse().expect("文件数");
        let units: usize = units.parse().expect("每个文件的单元数");
        for seed in seed_start..seed_start + seeds {
            run_scripted_history(files, units, seed, drawn_steps);
        }
    }
}
