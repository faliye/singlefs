//! 合法删除中间态上误不误红：用户决定的那几步放开扫（文件单元数、批大小、unlink 时机与对象、续做在进程里 / 会话里 / 挂载里、
//! 正常卸载再挂、管理员回退到哪一版、调用方给的时间、盘大小），每次发布或每次用户改动之后的整池镜像跑池级 checker。
//! 只固定前缀（mkfs → 取号 → 暖机 → 新池新建文件）；没有故障。
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish, unmount,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::{FirstFile, PoolVersion, TransactionOutput};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::DeletionSwitches;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

use crate::common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS};
use crate::judge::{judge, Judged};
use crate::pool::{batch, content_of, Dev, SparsePool};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    /// 意图与每一批都在这个进程里直接调发布。
    InProcess,
    /// 可写挂载之后经会话 `UserChange::Unlink`（走空 + 意图 + 全部批）。
    Session,
    /// 意图在进程里发，不卸载就重开可写挂载（进程退出），挂载续做走空。
    IntentThenRemount,
    /// 意图在进程里发，正常卸载，再可写挂载续做（H2 那一形）。
    IntentUnmountRemount,
    /// 进程里删完，管理员回退到删除那一串里第 k 版（0 = 意图之前那一版），再可写挂载续做。
    RollbackTo(usize),
    /// 进程里删到第 k 批之后停（不发后面的批），重开可写挂载续做。
    StopAfterBatchThenRemount(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum TimeChoice {
    SameAsRecord,
    Earlier,
    Later,
}

#[derive(Clone, Debug)]
pub struct History {
    pub device_bytes: u64,
    /// None：只有新池新建文件那 3000 字节；Some(n)：再顺序写 n 字节。
    pub sequential_write_bytes: Option<usize>,
    pub new_inodes_before: u64,
    pub targets: Vec<u64>,
    pub batch_units: u64,
    pub time: TimeChoice,
    pub mode: Mode,
    /// 删完之后再建几个空 inode、再删其中第一个（0 = 不做）。
    pub new_inodes_after_and_unlink_one: u64,
}

impl History {
    pub fn name(&self) -> String {
        format!(
            "dev{}G-seq{}-pre{}-t{:?}-b{}-{:?}-{:?}-post{}",
            self.device_bytes >> 30,
            self.sequential_write_bytes.map_or("none".to_string(), |bytes| bytes.to_string()),
            self.new_inodes_before,
            self.targets,
            self.batch_units,
            self.time,
            self.mode,
            self.new_inodes_after_and_unlink_one
        )
    }
}

pub fn switches(batch_units: u64) -> DeletionSwitches {
    DeletionSwitches { batch_size: batch(batch_units), ..DeletionSwitches::PRODUCT }
}

pub fn devices_of(image: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, Dev)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect()
}

pub fn image_of(devices: &[(DeviceIdentity, Dev)], device_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    }
}

fn time_for(choice: TimeChoice, record_time: u64) -> u64 {
    match choice {
        TimeChoice::SameAsRecord => record_time,
        TimeChoice::Earlier => record_time.saturating_sub(86_400),
        TimeChoice::Later => record_time + 86_400,
    }
}

fn record_time_of(output: &TransactionOutput, inode: u64) -> u64 {
    output
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == inode)
        .map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds)
}

/// 从 `images` 里取镜像的那一刻：会话与挂载走的路上每发成一次用户改动或一次挂载就取一份。
fn mount_and_continue(pool: &mut SparsePool, batch_units: u64, images: &mut Vec<(String, MemoryPool)>, label: &str) -> Result<MountedSession, String> {
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(batch_units),
    )
    .map_err(|error| format!("{label}：可写挂载被拒 {error:?}"))?;
    let (_output, session) = MountedSession::of_the_mount(mounted);
    images.push((format!("{label}之后"), pool.image()));
    Ok(session)
}

fn session_change(session: &mut MountedSession, pool: &mut SparsePool, change: UserChange<'_>, images: &mut Vec<(String, MemoryPool)>, label: String) -> Result<(), String> {
    session
        .publish_user_change(&mut pool.devices, change)
        .map_err(|refused| format!("{label}：会话拒了 {refused:?}"))?;
    images.push((label, pool.image()));
    Ok(())
}

/// 跑一段历史，交回每一次发布 / 用户改动 / 挂载之后的镜像（按先后）。
pub fn run_history(history: &History) -> Result<Vec<(String, MemoryPool)>, String> {
    let mut pool = SparsePool::build(history.device_bytes, &file_content());
    if let Some(bytes) = history.sequential_write_bytes {
        pool.sequential_write(&content_of(bytes), FIXED_WRITE_TIME_SECONDS + 60);
    }
    if history.new_inodes_before > 0 {
        pool.new_inodes(history.new_inodes_before, FIXED_WRITE_TIME_SECONDS + 120);
    }
    let mut images: Vec<(String, MemoryPool)> = Vec::new();
    let mut session: Option<MountedSession> = None;
    match history.mode {
        Mode::InProcess | Mode::RollbackTo(_) | Mode::StopAfterBatchThenRemount(_) => {
            let mut deletion_versions: Vec<TransactionOutput> = vec![pool.output.clone()];
            for (position, target) in history.targets.iter().enumerate() {
                let time = time_for(history.time, record_time_of(&pool.output, *target));
                let intent = pool.intent(*target, time).map_err(|error| format!("意图 {target}：{error:?}"))?;
                deletion_versions.push(intent);
                let mut batches_done = 0usize;
                while !pool.output.deleted_inodes.is_empty() {
                    if let Mode::StopAfterBatchThenRemount(stop) = history.mode {
                        if position + 1 == history.targets.len() && batches_done == stop {
                            break;
                        }
                    }
                    let inode = pool.output.deleted_inodes[0].0;
                    let step = pool.step(inode, history.batch_units).map_err(|error| format!("删除步：{error:?}"))?;
                    deletion_versions.push(step);
                    batches_done += 1;
                }
            }
            images.extend(pool.published_images.iter().cloned());
            if let Mode::RollbackTo(index) = history.mode {
                let target = deletion_versions.get(index).ok_or("回退目标越界")?.clone();
                let mut current = pool.output.clone();
                roll_back_by_a_forward_publish(
                    &parameters(),
                    &mut pool.devices,
                    &mut pool.allocator,
                    &mut current,
                    RollbackTarget { instance: target.root.instance, checkpoint_txg: target.root.checkpoint_txg },
                )
                .map_err(|error| format!("回退到第 {index} 版：{error:?}"))?;
                pool.output = current;
                images.push((format!("回退到第{index}版"), pool.image()));
                session = Some(mount_and_continue(&mut pool, history.batch_units, &mut images, "回退之后重开挂载")?);
            }
            if let Mode::StopAfterBatchThenRemount(_) = history.mode {
                session = Some(mount_and_continue(&mut pool, history.batch_units, &mut images, "停在半路重开挂载")?);
            }
        }
        Mode::Session => {
            images.extend(pool.published_images.iter().cloned());
            let mut opened = mount_and_continue(&mut pool, history.batch_units, &mut images, "挂载")?;
            for target in &history.targets {
                let time = time_for(history.time, FIXED_WRITE_TIME_SECONDS + 60);
                session_change(&mut opened, &mut pool, UserChange::Unlink { inode: InodeNumber(*target), write_time_seconds: time }, &mut images, format!("会话 unlink {target}"))?;
            }
            session = Some(opened);
        }
        Mode::IntentThenRemount | Mode::IntentUnmountRemount => {
            let first = history.targets[0];
            let time = time_for(history.time, record_time_of(&pool.output, first));
            pool.intent(first, time).map_err(|error| format!("意图：{error:?}"))?;
            images.extend(pool.published_images.iter().cloned());
            if let Mode::IntentUnmountRemount = history.mode {
                let mut current = PoolVersion::of_a_version_of_trees(pool.output.clone());
                unmount(&parameters(), &mut pool.devices, &mut pool.allocator, &mut current, ShadowLedger::On)
                    .map_err(|error| format!("卸载：{error:?}"))?;
                images.push(("卸载之后".to_string(), pool.image()));
            }
            let mut opened = mount_and_continue(&mut pool, history.batch_units, &mut images, "重开挂载续做")?;
            for target in &history.targets[1..] {
                let time = time_for(history.time, FIXED_WRITE_TIME_SECONDS + 60);
                session_change(&mut opened, &mut pool, UserChange::Unlink { inode: InodeNumber(*target), write_time_seconds: time }, &mut images, format!("会话 unlink {target}"))?;
            }
            session = Some(opened);
        }
    }
    if history.new_inodes_after_and_unlink_one > 0 {
        let mut opened = match session.take() {
            Some(opened) => opened,
            None => mount_and_continue(&mut pool, history.batch_units, &mut images, "删完重开挂载")?,
        };
        session_change(&mut opened, &mut pool, UserChange::NewInodes { count: history.new_inodes_after_and_unlink_one, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 300 }, &mut images, "会话建 inode".to_string())?;
        let watermark = record_highest_inode(&pool.image());
        session_change(&mut opened, &mut pool, UserChange::Unlink { inode: InodeNumber(watermark), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 400 }, &mut images, format!("会话 unlink 新建的 {watermark}"))?;
        session_change(&mut opened, &mut pool, UserChange::NewInodes { count: 1, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 500 }, &mut images, "会话再建 inode".to_string())?;
    }
    Ok(images)
}

/// 镜像里最新那一版 inode 树里号最大的那条记录（建了 inode 之后删它用）。
fn record_highest_inode(image: &MemoryPool) -> u64 {
    let mut largest = 0;
    let entry = crate::common_pointer_head_corruption::tree_table_entry_of_kind(image, singlefs_core::records::TREE_KIND_INODE);
    let root = singlefs_core::unit::parse_index_node(&crate::common_pointer_head_corruption::unit_bytes_at(
        image,
        crate::common_pointer_head_corruption::the_slot_of(&entry.root),
        crate::common_pointer_head_corruption::node_bytes(),
    ))
    .expect("inode 根");
    for child in &root.entries {
        let pointer = singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(child, 34));
        let leaf = crate::common_pointer_head_corruption::unit_bytes_at(image, pointer.locations[0].slot, crate::common_pointer_head_corruption::data_unit_bytes());
        let count = usize::from(u16::from_le_bytes([leaf[69], leaf[70]]));
        for index in 0..count {
            let start = 136 + index * 140;
            let inode = u64::from_le_bytes(leaf[start..start + 8].try_into().expect("8"));
            largest = largest.max(inode);
        }
    }
    largest
}

/// `extra` 为假：原先那 3060 段（时间与盘大小两轴只在进程里那一形扫满）；为真：补扫的那几段（别的形上时间与盘大小的其余取值），
/// 两份合起来是整张叉积，各自的序号不变。
pub fn histories(extra: bool) -> Vec<History> {
    let payload = data_unit_payload_capacity();
    let shapes: [Option<usize>; 6] = [None, Some(0), Some(payload), Some(2 * payload + payload / 2), Some(3 * payload), Some(5 * payload + 1)];
    let mut all = Vec::new();
    for device_bytes in [4u64 << 30, 8u64 << 30] {
        for shape in shapes {
            for new_inodes_before in [0u64, 2] {
                let target_lists: Vec<Vec<u64>> = if new_inodes_before == 0 { vec![vec![1]] } else { vec![vec![1], vec![2], vec![2, 1], vec![1, 3]] };
                for targets in target_lists {
                    for batch_units in [1u64, 2, 144] {
                        for time in [TimeChoice::SameAsRecord, TimeChoice::Earlier, TimeChoice::Later] {
                            let mut modes = vec![Mode::InProcess, Mode::Session, Mode::IntentThenRemount, Mode::IntentUnmountRemount];
                            for k in 0..4 {
                                modes.push(Mode::RollbackTo(k));
                                modes.push(Mode::StopAfterBatchThenRemount(k));
                            }
                            for mode in modes {
                                for post in [0u64, 2] {
                                    // 时间与盘大小：`extra` 为假的那一份只在进程里那一形上扫满，补扫那一份是其余的组合。
                                    let full_axes = matches!(mode, Mode::InProcess);
                                    let narrowed = !full_axes && (!matches!(time, TimeChoice::SameAsRecord) || device_bytes != 4 << 30);
                                    if narrowed != extra {
                                        continue;
                                    }
                                    all.push(History { device_bytes, sequential_write_bytes: shape, new_inodes_before, targets: targets.clone(), batch_units, time, mode, new_inodes_after_and_unlink_one: post });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    all
}

pub fn run(limit: Option<usize>, threads: usize) {
    let mut all = histories(std::env::var("M3B18_EXTRA").is_ok());
    if let Some(limit) = limit {
        all.truncate(limit);
    }
    // 分批跑：`M3B18_FROM` 起（含），`limit` 止（不含）；序号照整张表的。
    let from: usize = std::env::var("M3B18_FROM").ok().and_then(|n| n.parse().ok()).unwrap_or(0);
    let offset = from.min(all.len());
    let all: Vec<History> = all.split_off(offset);
    let started = std::time::Instant::now();
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(history) = all.get(index) else { break };
                let line = match run_history(history) {
                    Err(refused) => format!("SWEEP_HISTORY index={} name={} outcome=not_reachable reason={refused}", index + offset, history.name()),
                    Ok(images) => {
                        let mut red = Vec::new();
                        let mut watched_kinds = std::collections::BTreeMap::<String, u64>::new();
                        for (label, image) in &images {
                            let judged: Judged = judge(image);
                            for (invariant, kind, _) in &judged.watched {
                                *watched_kinds.entry(format!("{invariant}={kind}")).or_insert(0) += 1;
                            }
                            for (invariant, text) in &judged.violated {
                                red.push(format!("[{label}] {invariant}：{text}"));
                            }
                        }
                        format!(
                            "SWEEP_HISTORY index={} name={} outcome=ran images={} red={} watched={:?}{}",
                            index + offset,
                            history.name(),
                            images.len(),
                            red.len(),
                            watched_kinds,
                            if red.is_empty() { String::new() } else { format!(" first_red={}", red[0]) }
                        )
                    }
                };
                results.lock().expect("锁").push((index, line));
            });
        }
    });
    let mut lines = results.into_inner().expect("锁");
    lines.sort();
    let ran = lines.iter().filter(|(_, line)| line.contains("outcome=ran")).count();
    let unreachable = lines.iter().filter(|(_, line)| line.contains("outcome=not_reachable")).count();
    let red_histories = lines.iter().filter(|(_, line)| line.contains("outcome=ran") && !line.contains(" red=0 ")).count();
    let images: u64 = lines.iter().filter_map(|(_, line)| line.split(" images=").nth(1).and_then(|rest| rest.split(' ').next()).and_then(|n| n.parse::<u64>().ok())).sum();
    for (_, line) in &lines {
        println!("{line}");
    }
    println!(
        "SWEEP_SUMMARY histories={} ran={ran} not_reachable={unreachable} images_judged={images} red_histories={red_histories} threads={threads} elapsed_seconds={:.1}",
        lines.len(),
        started.elapsed().as_secs_f64()
    );
}
