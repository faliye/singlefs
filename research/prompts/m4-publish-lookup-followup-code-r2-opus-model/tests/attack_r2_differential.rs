//! 攻方腿（m4-publish-lookup-followup-code-r2）：改前（第 11 批的 transaction.rs 倒打回去）、改后两份副本各编这一份，按种子跑同一串随机的合法操作，
//! 每一步之后打一行：操作、结局、这一步录下的写流（写、FUA、清零、discard、屏障的次序与内容哈希）的 sha256、两块盘镜像的 sha256、这一版的摘要。
//! 写流逐条相同，两边的崩溃状态集合就相同。在第一轮攻方差分的操作之外加了：干净卸载再挂、不卸载直接重挂（两种都走恢复重建一版，
//! 调 take_the_mapping_positions_over_the_location_hints 的恢复那一侧）、截断、写第一个文件、一次写到 40 个单元（记录点名切几条）。
//! 种子范围与步数：ATTACK_SEED_FIRST、ATTACK_SEEDS、ATTACK_STEPS；线程 ATTACK_THREADS（默认 SINGLEFS_THREAD_CAP）。
mod common;

use std::collections::BTreeSet;
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{DeviceIdentity, InodeNumber, TreeIdentifier};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{CompactionBatchSize, CompactionBatchUnits, CompactionSwitches};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable_with_compaction_switches, unmount};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, RelocatedUnitHolder,
    TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

fn switches_with_the_batch_size(batch_units: usize) -> CompactionSwitches {
    let mut switches = CompactionSwitches::PRODUCT;
    switches.batch_size = CompactionBatchSize::ForcedForTests(
        CompactionBatchUnits::new(batch_units).expect("批大小至少 1"),
    );
    switches
}

fn formatted_and_mounted(switches: CompactionSwitches, stream: &SharedStream) -> (Devices, MountedSession) {
    let parameters = common::parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let device = SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))
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

fn hex12(bytes: &[u8]) -> String {
    singlefs_harness::sha256::sha256_digest(bytes).iter().take(12).map(|byte| format!("{byte:02x}")).collect()
}

fn digest_of_the_images(devices: &Devices) -> String {
    let mut written = Vec::new();
    for (identity, device) in devices {
        written.extend_from_slice(&identity.0.to_le_bytes());
        for (sector, bytes) in device.wrapped_device().image.written_sectors() {
            written.extend_from_slice(&sector.to_le_bytes());
            written.extend_from_slice(bytes);
        }
    }
    hex12(&written)
}

/// 写流里从 `from` 起的那一段（这一步录下的）：条数与逐条文本的 sha256。
fn digest_of_the_stream_since(stream: &SharedStream, from: usize) -> (usize, String) {
    let operations = stream.operations();
    let mut text = String::new();
    for operation in &operations[from.min(operations.len())..] {
        text.push_str(&operation.to_stream_line());
        text.push('\n');
    }
    (operations.len().saturating_sub(from), hex12(text.as_bytes()))
}

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

fn content(data_units: usize, seed: u64, short_by: usize) -> Vec<u8> {
    let mut random = Random(seed | 1);
    (0..(data_units * data_unit_payload_capacity()).saturating_sub(short_by).max(1))
        .map(|_| u8::try_from(random.next() & 0xff).expect("低 8 位"))
        .collect()
}

fn version_of(session: &MountedSession) -> Option<&TransactionOutput> {
    session.current.version_of_trees()
}

fn summary_of(session: &MountedSession) -> String {
    let Some(version) = version_of(session) else {
        return "no-trees".to_string();
    };
    let kept = &version.mapping_entries_kept_for_the_snapshots;
    let distinct_kept: BTreeSet<&[u8]> = kept.iter().map(|(key, _)| key.as_slice()).collect();
    let roles: BTreeSet<_> = version.units.iter().map(|unit| unit.identity).collect();
    let slots: Vec<u64> = version.units.iter().map(|unit| unit.slot.0).collect();
    format!(
        "kept={} dupkept={} files={} units={} dupunits={} snapshots={} slots={}",
        kept.len(),
        kept.len() - distinct_kept.len(),
        version.files_other_than_the_first.len(),
        version.units.len(),
        version.units.len() - roles.len(),
        live_snapshots(version).len(),
        hex12(&slots.iter().flat_map(|slot| slot.to_le_bytes()).collect::<Vec<u8>>())
    )
}

fn live_snapshots(version: &TransactionOutput) -> Vec<TreeIdentifier> {
    version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .filter(|tree| !version.snapshots_being_destroyed.contains(tree))
        .collect()
}

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|text| text.parse().ok()).unwrap_or(default)
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
        .unwrap_or_else(|| "<非文本>".to_string())
}

fn outcome_of<T, E: std::fmt::Debug>(result: &Result<T, E>) -> String {
    match result {
        Ok(_) => "ok".to_string(),
        Err(error) => {
            let text = format!("{error:?}");
            let head: String = text.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            format!("err:{head}:{}", text.len())
        }
    }
}

fn pick(random: &mut Random, inodes: &[InodeNumber]) -> InodeNumber {
    inodes[usize::try_from(random.below(inodes.len() as u64)).expect("下标")]
}

#[allow(clippy::too_many_lines, reason = "攻方模型：一个 match 列全部操作")]
fn run_one_history(seed: u64, steps: u64) -> Vec<String> {
    let mut lines = Vec::new();
    let mut random = Random(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);
    let batch = [1usize, 2, 3, 8][usize::try_from(random.below(4)).expect("下标")];
    let parameters = common::parameters();
    let stream = SharedStream::new();
    let (mut devices, mut session) = formatted_and_mounted(switches_with_the_batch_size(batch), &stream);
    let mut write_time = common::FIXED_WRITE_TIME_SECONDS + 600;
    let mut inodes: Vec<InodeNumber> = Vec::new();
    let mut kept_relocated_total = 0usize;
    let mut remounts = 0usize;
    lines.push(format!("seed={seed} batch={batch} start {}", digest_of_the_images(&devices)));
    for step in 0..steps {
        write_time += 1;
        let stream_before = stream.operation_count();
        let choice = if step < 2 { 0 } else { random.below(100) };
        let live = version_of(&session).map(live_snapshots).unwrap_or_default();
        let (label, outcome): (String, Result<String, Box<dyn std::any::Any + Send>>) = match choice {
            0..=6 => {
                let count = 1 + random.below(6);
                let first = version_of(&session).map_or(0, TransactionOutput::inode_number_watermark);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::NewInodes { count, write_time_seconds: write_time })
                }));
                if matches!(result, Ok(Ok(_))) {
                    inodes.extend((first..first + count).map(InodeNumber));
                }
                (format!("new_inodes({count})"), result.map(|r| outcome_of(&r)))
            }
            7..=34 if !inodes.is_empty() => {
                let inode = pick(&mut random, &inodes);
                let units = if random.below(5) == 0 { 10 + usize::try_from(random.below(31)).expect("单元数") } else { 1 + usize::try_from(random.below(9)).expect("单元数") };
                let bytes = content(units, random.next(), usize::try_from(random.below(7)).expect("短"));
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode, content: &bytes, write_time_seconds: write_time })
                }));
                (format!("write({},{units})", inode.0), result.map(|r| outcome_of(&r)))
            }
            35..=39 if !inodes.is_empty() => {
                let inode = pick(&mut random, &inodes);
                let bytes = content(1, random.next(), 1 + usize::try_from(random.below(300)).expect("短"));
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::OverwriteOfAnInode { inode, content: &bytes, write_time_seconds: write_time })
                }));
                (format!("overwrite({})", inode.0), result.map(|r| outcome_of(&r)))
            }
            40..=44 => {
                let units = 1 + usize::try_from(random.below(12)).expect("单元数");
                let bytes = content(units, random.next(), usize::try_from(random.below(7)).expect("短"));
                let sequential = random.below(2) == 0;
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let file = FirstFile { content: &bytes, write_time_seconds: write_time };
                    let change = if sequential { UserChange::SequentialWrite(file) } else { UserChange::Overwrite(file) };
                    session.publish_user_change(&mut devices, change)
                }));
                (format!("first_file({units},{sequential})"), result.map(|r| outcome_of(&r)))
            }
            45..=48 => {
                // 截断只对第一个文件有条款（别的 inode 拒 SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses）。
                let inode = InodeNumber(singlefs_core::transaction::FIRST_INODE_NUMBER);
                let new_size = random.below(6 * data_unit_payload_capacity() as u64);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::Truncate { inode, new_size, write_time_seconds: write_time })
                }));
                (format!("truncate({},{new_size})", inode.0), result.map(|r| outcome_of(&r)))
            }
            49..=56 if live.len() < 3 => {
                let result = catch_unwind(AssertUnwindSafe(|| session.publish_user_change(&mut devices, UserChange::CreateSnapshot)));
                ("create_snapshot".to_string(), result.map(|r| outcome_of(&r)))
            }
            57..=62 if !live.is_empty() => {
                let snapshot = live[usize::try_from(random.below(live.len() as u64)).expect("下标")];
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::DestroySnapshot { snapshot })
                }));
                (format!("destroy_snapshot({})", snapshot.0), result.map(|r| outcome_of(&r)))
            }
            63..=65 if inodes.len() > 2 => {
                let position = usize::try_from(random.below(inodes.len() as u64)).expect("下标");
                let inode = inodes.remove(position);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::Unlink { inode, write_time_seconds: write_time })
                }));
                (format!("unlink({})", inode.0), result.map(|r| outcome_of(&r)))
            }
            66..=82 => {
                let result = catch_unwind(AssertUnwindSafe(|| session.run_compaction(&mut devices)));
                let text = match &result {
                    Ok(Ok(run)) => {
                        let kept: usize = run
                            .versions_published
                            .iter()
                            .flat_map(|version| version.units_relocated_by_this_publish.iter())
                            .filter(|unit| unit.holder == RelocatedUnitHolder::KeptForTheSnapshots)
                            .count();
                        kept_relocated_total += kept;
                        format!("ok:batches={}:kept_relocated={kept}:{:?}", run.batches_published, std::mem::discriminant(&run.outcome))
                    }
                    Ok(error) => outcome_of(error),
                    Err(_) => String::new(),
                };
                ("compaction".to_string(), result.map(|_| text))
            }
            _ => {
                // 重挂：一半先干净卸载，一半不卸载直接挂（恢复重放 journal）。两种都从盘上重建一版。
                let clean = random.below(2) == 0;
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let unmounted = if clean {
                        outcome_of(&unmount(&parameters, &mut devices, &mut session.allocator, &mut session.current, session.shadow_ledger))
                    } else {
                        "skipped".to_string()
                    };
                    match mount_writable_with_compaction_switches(&parameters, &mut devices, switches_with_the_batch_size(batch)) {
                        Ok(mounted) => {
                            session = MountedSession::of_the_mount(mounted).1;
                            remounts += 1;
                            format!("unmount={unmounted}:mount=ok")
                        }
                        Err(error) => format!("unmount={unmounted}:mount={}", outcome_of::<(), _>(&Err(error))),
                    }
                }));
                (format!("remount({clean})"), result)
            }
        };
        let (stream_steps, stream_digest) = digest_of_the_stream_since(&stream, stream_before);
        match outcome {
            Ok(text) => lines.push(format!(
                "seed={seed} step={step} {label} {text} stream={stream_steps}:{stream_digest} {} {}",
                digest_of_the_images(&devices),
                summary_of(&session)
            )),
            Err(payload) => {
                lines.push(format!("seed={seed} step={step} {label} panic:{}", panic_text(&*payload)));
                return lines;
            }
        }
        if lines.last().is_some_and(|line| line.contains("mount=err")) {
            return lines;
        }
    }
    lines.push(format!("seed={seed} end kept_relocated_total={kept_relocated_total} remounts={remounts}"));
    lines
}

#[test]
fn attack_r2_differential_history() {
    let first = environment_number("ATTACK_SEED_FIRST", 1);
    let seeds = environment_number("ATTACK_SEEDS", 4);
    let steps = environment_number("ATTACK_STEPS", 40);
    let threads = environment_number("ATTACK_THREADS", environment_number("SINGLEFS_THREAD_CAP", 1)).max(1);
    let all_seeds: Vec<u64> = (first..first + seeds).collect();
    let mut outputs: Vec<(u64, Vec<String>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|worker| {
                let mine: Vec<u64> = all_seeds
                    .iter()
                    .copied()
                    .skip(usize::try_from(worker).expect("线程号"))
                    .step_by(usize::try_from(threads).expect("线程数"))
                    .collect();
                scope.spawn(move || mine.into_iter().map(|seed| (seed, run_one_history(seed, steps))).collect::<Vec<_>>())
            })
            .collect();
        workers.into_iter().flat_map(|worker| worker.join().expect("一条线程没 panic 出来")).collect()
    });
    outputs.sort_by_key(|(seed, _)| *seed);
    for (_, lines) in outputs {
        for line in lines {
            println!("{line}");
        }
    }
}
