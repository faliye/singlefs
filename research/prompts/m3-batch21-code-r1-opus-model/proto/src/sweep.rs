//! 合法历史（没有故障）放开扫用户决定的几步，每次发布、挂载之后的整池镜像跑池级 checker，另记 CPU 两遍恢复结局与 GPU 恢复内核会拒的形状。
//! 与第 20 批攻方那一份取的轴不同：删除进行到一半时建 inode、删完再删刚建的、挂载续做从意图之后与第 j 批之后起、批大小取 1、2、3、144。
use std::collections::BTreeMap;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{mount_writable_with_test_only_switches_and_deletion_switches, ShadowLedger};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::DeletionSwitches;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

use crate::judge::{judge, recovered};
use crate::pool::{batch, content_of, parameters, SparsePool};

#[derive(Clone, Debug)]
pub struct History {
    pub units: usize,
    pub inodes_before: u64,
    /// 删除的次序：0 表示「最后建的那个」，1 是第一个文件。
    pub order: Vec<u64>,
    pub batch: u64,
    /// 第一个号删到第 j 批之后（0 是意图之后）做的那一步：0 不做、1 建一个 inode、2 不卸载重挂续做（历史到此为止）。
    pub middle_after_batch: usize,
    pub middle: u8,
    /// 删完之后再建 2 个、删掉号最大的那个。
    pub post: bool,
}

impl History {
    pub fn name(&self) -> String {
        format!(
            "u{}-n{}-o{:?}-b{}-mid{}@{}-post{}",
            self.units, self.inodes_before, self.order, self.batch, self.middle, self.middle_after_batch, u8::from(self.post)
        )
    }
}

pub fn histories() -> Vec<History> {
    let mut all = Vec::new();
    for units in [1usize, 3, 145] {
        for inodes_before in [0u64, 1, 3] {
            let orders: Vec<Vec<u64>> = if inodes_before == 0 { vec![vec![1]] } else { vec![vec![1], vec![0, 1], vec![1, 0]] };
            for order in orders {
                let batches: Vec<u64> = if units == 145 { vec![100, 144] } else { vec![1, 2, 144] };
                for batch in batches {
                    for (middle, middle_after_batch) in [(0u8, 0usize), (1, 0), (1, 1), (2, 0), (2, 1)] {
                        for post in [false, true] {
                            if middle == 2 && post {
                                continue;
                            }
                            all.push(History { units, inodes_before, order: order.clone(), batch, middle_after_batch, middle, post });
                        }
                    }
                }
            }
        }
    }
    // 多片 inode 叶：建 240、470 个空 inode（一片叶容器装 233 条左右），删 1 号或最后建的那个再删 1 号，删到一半建一个。
    for inodes_before in [240u64, 470] {
        for order in [vec![1u64], vec![0, 1], vec![1, 0]] {
            for (middle, middle_after_batch) in [(0u8, 0usize), (1, 0), (2, 0)] {
                all.push(History { units: 3, inodes_before, order: order.clone(), batch: 1, middle_after_batch, middle, post: middle != 2 });
            }
        }
    }
    all
}

pub struct Outcome {
    pub images: Vec<(String, MemoryPool)>,
    pub note: String,
}

fn devices_on(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

fn remount_and_continue(image: &MemoryPool, units_per_batch: u64) -> Result<MemoryPool, String> {
    let mut devices = devices_on(image);
    let switches = DeletionSwitches {
        batch_size: batch(units_per_batch),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    mount_writable_with_test_only_switches_and_deletion_switches(&parameters(), &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches)
        .map_err(|error| format!("{error:?}").chars().take(120).collect::<String>())?;
    Ok(MemoryPool {
        devices: devices.into_iter().map(|(identity, device)| (identity, device.image)).collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    })
}

pub fn run_history(history: &History) -> Outcome {
    let payload = data_unit_payload_capacity();
    let mut pool = SparsePool::build(4 << 30, &content_of(3000, 0));
    if history.units > 1 {
        pool.sequential_write(&content_of((history.units - 1) * payload + payload / 2, 1));
    }
    if history.inodes_before > 0 {
        pool.new_inodes(history.inodes_before);
    }
    let last_created = 1 + history.inodes_before;
    let mut note = String::new();
    let mut extra: Vec<(String, MemoryPool)> = Vec::new();
    'outer: for (position, target) in history.order.iter().enumerate() {
        let inode = if *target == 0 { last_created } else { *target };
        if let Err(error) = pool.intent(inode) {
            note = format!("意图{inode}拒:{error:?}");
            break;
        }
        let mut batches_done = 0usize;
        loop {
            if position == 0 && batches_done == history.middle_after_batch && !pool.queue_is_empty() {
                match history.middle {
                    1 => pool.new_inodes(1),
                    2 => {
                        match remount_and_continue(&pool.image(), history.batch) {
                            Ok(image) => extra.push((format!("重挂续做@删除{inode}第{batches_done}批后"), image)),
                            Err(error) => note = format!("重挂拒:{error}"),
                        }
                        break 'outer;
                    }
                    _ => {}
                }
            }
            if pool.queue_is_empty() {
                break;
            }
            if let Err(error) = pool.step(history.batch) {
                note = format!("删除步拒:{error:?}");
                break 'outer;
            }
            batches_done += 1;
            assert!(batches_done < 10_000, "走不空");
        }
    }
    if history.post && note.is_empty() {
        pool.new_inodes(2);
        let largest = pool.inode_numbers().into_iter().max().expect("有号");
        if pool.intent(largest).is_ok() {
            while !pool.queue_is_empty() {
                pool.step(history.batch).expect("删除步");
            }
        }
    }
    let mut images: Vec<(String, MemoryPool)> = pool.published.drain(..).map(|published| (published.name, published.image)).collect();
    images.extend(extra);
    Outcome { images, note }
}

pub fn run(from: usize, to: usize, threads: usize) {
    let all = histories();
    let chosen: Vec<(usize, History)> = all.into_iter().enumerate().filter(|(index, _)| *index >= from && *index < to).collect();
    let chunks: Vec<Vec<(usize, History)>> = (0..threads).map(|t| chosen.iter().filter(|(index, _)| index % threads == t).cloned().collect()).collect();
    let mut lines: Vec<(usize, String)> = std::thread::scope(|scope| {
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .into_iter()
                        .map(|(index, history)| (index, judge_history(index, &history)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|handle| handle.join().expect("线程")).collect()
    });
    lines.sort();
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    for (_, line) in &lines {
        println!("{line}");
        for field in line.split(' ').filter(|field| field.starts_with("tally:")) {
            for pair in field.trim_start_matches("tally:").split(',').filter(|pair| !pair.is_empty()) {
                let (key, count) = pair.split_once('=').expect("k=v");
                *totals.entry(key.to_string()).or_default() += count.parse::<u64>().expect("数");
            }
        }
    }
    println!("SWEEP_TOTALS histories={} {:?}", lines.len(), totals);
}

fn judge_history(index: usize, history: &History) -> String {
    let outcome = run_history(history);
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    let mut flagged = Vec::new();
    for (name, image) in &outcome.images {
        let judged = judge(image);
        for (invariant, kind, text) in &judged.watched {
            *tally.entry(format!("{invariant}{kind}")).or_default() += 1;
            if matches!(kind, 'G' | 'P' | 'V') {
                flagged.push(format!("[{name}:{invariant}={kind}:{}]", text.chars().take(100).collect::<String>()));
            }
        }
        for (invariant, text) in &judged.violated {
            if !crate::judge::WATCHED.contains(invariant) {
                *tally.entry("other_violation".to_string()).or_default() += 1;
                flagged.push(format!("[{name}:{invariant}=V:{}]", text.chars().take(100).collect::<String>()));
            }
        }
        let (consulted, ignored, shape) = recovered(image);
        *tally.entry(format!("cpu{consulted}{ignored}").chars().take(12).collect()).or_default() += 1;
        if !shape.is_empty() {
            *tally.entry("gpu_shape".to_string()).or_default() += 1;
            flagged.push(format!("[{name}:gpu:{consulted}{ignored}:{shape}]"));
        }
        *tally.entry("images".to_string()).or_default() += 1;
    }
    let tally_text: Vec<String> = tally.iter().map(|(key, count)| format!("{key}={count}")).collect();
    format!(
        "SWEEP_HISTORY index={index} name={} note={} tally:{} flagged={}",
        history.name(),
        if outcome.note.is_empty() { "-".to_string() } else { outcome.note.replace(' ', "_") },
        tally_text.join(","),
        flagged.join("").replace(' ', "_")
    )
}
