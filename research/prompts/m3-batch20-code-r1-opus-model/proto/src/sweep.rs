//! N1 与 N5 合在一遍：用户决定的那几步放开扫（文件单元数、建 inode 的个数与先后、unlink 对象与次序、批大小、续做在进程里 / 会话里 /
//! 挂载里、正常卸载再挂、管理员回退到哪一版、调用方时间、盘大小、会话的容器与节点容量），只固定前缀（mkfs → 取号 → 暖机 → 新池新建文件），
//! 没有故障。每次发布、每次用户改动、每次挂载之后的整池镜像：池级 checker 判（误红、误报判不了），两种 journal 政策的 CPU 恢复
//! （GPU 恢复内核要给的期望），最新那条根的 extent 树根条目数（GPU `judge_extent_node` 拒零条目的那一支）。
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::{ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::tombstone_tree::{TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests};
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::DeletionSwitches;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

use crate::common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS};
use crate::judge::{judge, recovered};
use crate::pool::{batch, content_of, Dev, SparsePool};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    InProcess,
    Session,
    IntentThenRemount,
    IntentUnmountRemount,
    RollbackTo(usize),
    StopAfterBatchThenRemount(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub new_inodes: u64,
    /// 空 inode 建在顺序写之前（H3 的次序）还是之后。
    pub inodes_before_the_write: bool,
    pub targets: Vec<u64>,
    pub batch_units: u64,
    pub time: TimeChoice,
    pub mode: Mode,
    /// 删完之后再建几个空 inode、删其中号最大的那个、再建一个（0 = 不做）。
    pub post: u64,
    /// 会话里墓碑容器 1 条、容器树节点 2 条（逼出多片容器、多层墓碑树）。
    pub small_containers: bool,
}

impl History {
    pub fn name(&self) -> String {
        format!(
            "dev{}G-seq{}-ino{}{}-t{:?}-b{}-{:?}-{:?}-post{}{}",
            self.device_bytes >> 30,
            self.sequential_write_bytes.map_or("none".to_string(), |bytes| bytes.to_string()),
            self.new_inodes,
            if self.inodes_before_the_write { "first" } else { "after" },
            self.targets,
            self.batch_units,
            self.time,
            self.mode,
            self.post,
            if self.small_containers { "-small" } else { "" }
        )
    }
}

pub fn switches(batch_units: u64, small: bool) -> DeletionSwitches {
    let mut switches = DeletionSwitches { batch_size: batch(batch_units), ..DeletionSwitches::PRODUCT };
    if small {
        switches.tombstone_container_capacity =
            TombstoneContainerCapacity::CappedForTests(TombstoneRecordsPerContainerCappedForTests::new(1).expect("1"));
        switches.container_tree_node_entries =
            ContainerTreeNodeEntries::CappedForTests(ContainerTreeNodeEntriesCappedForTests::new(2).expect("2"));
    }
    switches
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

fn mount_and_continue(pool: &mut SparsePool, history: &History, images: &mut Vec<(String, MemoryPool)>, label: &str) -> Result<MountedSession, String> {
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(history.batch_units, history.small_containers),
    )
    .map_err(|error| format!("{label}：可写挂载被拒 {error:?}"))?;
    let (_output, session) = MountedSession::of_the_mount(mounted);
    images.push((format!("{label}之后"), pool.image()));
    Ok(session)
}

fn session_change(session: &mut MountedSession, pool: &mut SparsePool, change: UserChange<'_>, images: &mut Vec<(String, MemoryPool)>, label: String) -> Result<(), String> {
    session.publish_user_change(&mut pool.devices, change).map_err(|refused| format!("{label}：会话拒了 {refused:?}"))?;
    images.push((label, pool.image()));
    Ok(())
}

/// 跑一段历史，交回每一次发布 / 用户改动 / 挂载之后的镜像（按先后）。
pub fn run_history(history: &History) -> Result<Vec<(String, MemoryPool)>, String> {
    let mut pool = SparsePool::build(history.device_bytes, &file_content());
    if history.inodes_before_the_write && history.new_inodes > 0 {
        pool.new_inodes(history.new_inodes, FIXED_WRITE_TIME_SECONDS);
    }
    if let Some(bytes) = history.sequential_write_bytes {
        pool.sequential_write(&content_of(bytes), FIXED_WRITE_TIME_SECONDS + 60);
    }
    if !history.inodes_before_the_write && history.new_inodes > 0 {
        pool.new_inodes(history.new_inodes, FIXED_WRITE_TIME_SECONDS + 120);
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
                session = Some(mount_and_continue(&mut pool, history, &mut images, "回退之后重开挂载")?);
            }
            if let Mode::StopAfterBatchThenRemount(_) = history.mode {
                session = Some(mount_and_continue(&mut pool, history, &mut images, "停在半路重开挂载")?);
            }
        }
        Mode::Session => {
            images.extend(pool.published_images.iter().cloned());
            let mut opened = mount_and_continue(&mut pool, history, &mut images, "挂载")?;
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
                unmount(&parameters(), &mut pool.devices, &mut pool.allocator, &mut current, ShadowLedger::On).map_err(|error| format!("卸载：{error:?}"))?;
                images.push(("卸载之后".to_string(), pool.image()));
            }
            let mut opened = mount_and_continue(&mut pool, history, &mut images, "重开挂载续做")?;
            for target in &history.targets[1..] {
                let time = time_for(history.time, FIXED_WRITE_TIME_SECONDS + 60);
                session_change(&mut opened, &mut pool, UserChange::Unlink { inode: InodeNumber(*target), write_time_seconds: time }, &mut images, format!("会话 unlink {target}"))?;
            }
            session = Some(opened);
        }
    }
    if history.post > 0 {
        let mut opened = match session.take() {
            Some(opened) => opened,
            None => mount_and_continue(&mut pool, history, &mut images, "删完重开挂载")?,
        };
        session_change(&mut opened, &mut pool, UserChange::NewInodes { count: history.post, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 300 }, &mut images, "会话建 inode".to_string())?;
        let highest = highest_inode_of(&pool.image());
        session_change(&mut opened, &mut pool, UserChange::Unlink { inode: InodeNumber(highest), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 400 }, &mut images, format!("会话 unlink 新建的 {highest}"))?;
        session_change(&mut opened, &mut pool, UserChange::NewInodes { count: 1, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 500 }, &mut images, "会话再建 inode".to_string())?;
    }
    Ok(images)
}

/// 最新那一版 inode 树里号最大的那条记录。
fn highest_inode_of(image: &MemoryPool) -> u64 {
    use crate::common_pointer_head_corruption::{data_unit_bytes, node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at};
    let entry = tree_table_entry_of_kind(image, singlefs_core::records::TREE_KIND_INODE);
    let root = singlefs_core::unit::parse_index_node(&unit_bytes_at(image, the_slot_of(&entry.root), node_bytes())).expect("inode 根");
    let mut largest = 0;
    for child in &root.entries {
        let pointer = singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(child, 34));
        let leaf = unit_bytes_at(image, pointer.locations[0].slot, data_unit_bytes());
        let count = usize::from(u16::from_le_bytes([leaf[69], leaf[70]]));
        for index in 0..count {
            let start = 136 + index * 140;
            largest = largest.max(u64::from_le_bytes(leaf[start..start + 8].try_into().expect("8")));
        }
    }
    largest
}

/// 整张历史表（序号固定）。轴：7 种文件形状 × 5 种空 inode 配置（各带几种 unlink 对象与次序）× 批大小 × 11 种续做 / 回退方式；
/// post（删完再建再删）只在进程里与会话里两种上加，会话小容器只在会话里加，调用方时间早晚与 8 GiB 盘只在进程里、2.5 净荷那一形上加。
pub fn histories() -> Vec<History> {
    let payload = data_unit_payload_capacity();
    let two_and_a_half = 2 * payload + payload / 2;
    let shapes: [Option<usize>; 7] = [None, Some(0), Some(payload), Some(two_and_a_half), Some(3 * payload), Some(5 * payload + 1), Some(145 * payload)];
    let inode_configurations: Vec<(u64, bool, Vec<Vec<u64>>)> = vec![
        (0, false, vec![vec![1]]),
        (1, true, vec![vec![1], vec![2], vec![2, 1], vec![1, 2]]),
        (1, false, vec![vec![1], vec![2], vec![1, 2]]),
        (2, false, vec![vec![1], vec![2], vec![2, 1], vec![1, 3], vec![3, 2, 1]]),
        (300, false, vec![vec![1], vec![301], vec![150, 1]]),
    ];
    let mut modes = vec![Mode::InProcess, Mode::Session, Mode::IntentThenRemount, Mode::IntentUnmountRemount];
    for k in 0..4 {
        modes.push(Mode::RollbackTo(k));
    }
    for k in 0..3 {
        modes.push(Mode::StopAfterBatchThenRemount(k));
    }
    let mut all = Vec::new();
    for shape in shapes {
        let batches: Vec<u64> = if shape == Some(145 * payload) { vec![144, 100] } else { vec![1, 2, 144] };
        for (new_inodes, inodes_before_the_write, target_lists) in &inode_configurations {
            for targets in target_lists {
                for batch_units in &batches {
                    for mode in &modes {
                        let base = History {
                            device_bytes: 4 << 30,
                            sequential_write_bytes: shape,
                            new_inodes: *new_inodes,
                            inodes_before_the_write: *inodes_before_the_write,
                            targets: targets.clone(),
                            batch_units: *batch_units,
                            time: TimeChoice::SameAsRecord,
                            mode: *mode,
                            post: 0,
                            small_containers: false,
                        };
                        all.push(base.clone());
                        if matches!(mode, Mode::InProcess | Mode::Session) {
                            all.push(History { post: 2, ..base.clone() });
                        }
                        if matches!(mode, Mode::Session) {
                            all.push(History { small_containers: true, ..base.clone() });
                        }
                        if matches!(mode, Mode::InProcess) && shape == Some(two_and_a_half) {
                            all.push(History { time: TimeChoice::Earlier, ..base.clone() });
                            all.push(History { time: TimeChoice::Later, ..base.clone() });
                            all.push(History { device_bytes: 8 << 30, ..base.clone() });
                        }
                    }
                }
            }
        }
    }
    all
}

pub fn run(from: usize, to: usize, threads: usize) {
    let all = histories();
    let end = to.min(all.len());
    let slice: Vec<History> = all[from.min(end)..end].to_vec();
    println!("SWEEP_BATCH from={from} to={end} of={}", all.len());
    let started = std::time::Instant::now();
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(history) = slice.get(index) else { break };
                let line = match run_history(history) {
                    Err(refused) => format!("SWEEP_HISTORY index={} name={} outcome=not_reachable reason={}", index + from, history.name(), refused.lines().next().unwrap_or("")),
                    Ok(images) => describe(index + from, history, &images),
                };
                results.lock().expect("锁").push((index, line));
            });
        }
    });
    let mut lines = results.into_inner().expect("锁");
    lines.sort();
    for (_, line) in &lines {
        println!("{line}");
    }
    let count = |needle: &str| lines.iter().filter(|(_, line)| line.contains(needle)).count();
    println!(
        "SWEEP_SUMMARY from={from} to={end} histories={} ran={} not_reachable={} red_histories={} gap_histories={} gpu_extent_zero_entry_no_file_histories={} threads={threads} elapsed_seconds={:.1}",
        lines.len(),
        count("outcome=ran"),
        count("outcome=not_reachable"),
        lines.iter().filter(|(_, line)| line.contains("outcome=ran") && !line.contains(" red=0 ")).count(),
        lines.iter().filter(|(_, line)| line.contains("outcome=ran") && !line.contains(" gap=0 ")).count(),
        lines.iter().filter(|(_, line)| line.contains("outcome=ran") && !line.contains(" gpu_divergent=0 ")).count(),
        started.elapsed().as_secs_f64()
    );
}

/// 一段历史的一行：镜像数、误红、误报判不了（G / P）、六条判定种类计数、CPU 恢复结局计数、GPU 那一支会分歧的镜像数与第一处。
fn describe(index: usize, history: &History, images: &[(String, MemoryPool)]) -> String {
    let mut red = Vec::new();
    let mut gap = Vec::new();
    let mut watched_kinds = std::collections::BTreeMap::<String, u64>::new();
    let mut outcomes = std::collections::BTreeMap::<String, u64>::new();
    let mut gpu_divergent = Vec::new();
    let mut not_newest = 0;
    for (label, image) in images {
        let judged = judge(image);
        for (invariant, kind, detail) in &judged.watched {
            *watched_kinds.entry(format!("{invariant}={kind}")).or_insert(0) += 1;
            if *kind == 'G' || *kind == 'P' {
                gap.push(format!("[{label}] {invariant}={kind}：{detail}"));
            }
        }
        for (invariant, text) in &judged.violated {
            red.push(format!("[{label}] {invariant}：{text}"));
        }
        let recovered = recovered(image);
        if !recovered.effective_is_newest {
            not_newest += 1;
        }
        let consult_class = recovered.consulted.chars().next().unwrap_or('?');
        let ignore_class = recovered.ignored.chars().next().unwrap_or('?');
        *outcomes.entry(format!("{consult_class}{ignore_class}")).or_insert(0) += 1;
        // GPU `read_upper_segment` → `judge_extent_node` 对零条目的节点交回假，`walk_to_file` 报 walk_failed(32)；CPU 认零条目的上段根。
        if recovered.extent_root_entries == Some(0) && (consult_class != 'X' || ignore_class != 'X') {
            gpu_divergent.push(format!(
                "[{label}] cpu_consulted={} cpu_ignored={} extent_root={} inode_root={}",
                recovered.consulted, recovered.ignored, recovered.extent_root, recovered.inode_root
            ));
        }
    }
    format!(
        "SWEEP_HISTORY index={index} name={} outcome=ran images={} red={} gap={} gpu_divergent={} not_newest_effective_root={not_newest} watched={:?} cpu_outcomes={:?}{}{}{}",
        history.name(),
        images.len(),
        red.len(),
        gap.len(),
        gpu_divergent.len(),
        watched_kinds,
        outcomes,
        red.first().map_or(String::new(), |first| format!(" first_red={first}")),
        gap.first().map_or(String::new(), |first| format!(" first_gap={first}")),
        gpu_divergent.first().map_or(String::new(), |first| format!(" first_gpu_divergent={first}")),
    )
}

/// 估挂钟用：整张表里每隔 `stride` 取一段跑，报每段的平均耗时与按全表推的总挂钟。
pub fn estimate(stride: usize, threads: usize) {
    let all = histories();
    let picked: Vec<(usize, History)> = all.iter().cloned().enumerate().step_by(stride.max(1)).collect();
    let started = std::time::Instant::now();
    let next = AtomicUsize::new(0);
    let images = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some((_, history)) = picked.get(index) else { break };
                if let Ok(run) = run_history(history) {
                    let _ = describe(0, history, &run);
                    images.fetch_add(run.len(), Ordering::SeqCst);
                }
            });
        }
    });
    let elapsed = started.elapsed().as_secs_f64();
    println!(
        "SWEEP_ESTIMATE histories_total={} sampled={} images_sampled={} elapsed_seconds={elapsed:.1} threads={threads} projected_total_seconds={:.0}",
        all.len(),
        picked.len(),
        images.load(Ordering::SeqCst),
        elapsed * all.len() as f64 / picked.len() as f64
    );
}
