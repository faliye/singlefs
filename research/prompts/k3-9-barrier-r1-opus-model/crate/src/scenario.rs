//! 起点历史：在内存稀疏盘上跑 mkfs、挂载与若干次发布，交回被判那一次发布之前的池、它的录制流与版本表。
#![allow(dead_code)]

use std::collections::BTreeSet;

use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable_with_space_admission, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_sequential_write, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, PublishedVersion};
use singlefs_harness::RetainedOperation;

use crate::common::{Key, Session};

pub const WRITE_TIME: u64 = 1_788_000_000;

pub fn content(units: usize, seed: u8, small: usize) -> Vec<u8> {
    let capacity = data_unit_payload_capacity();
    let length = if units == 1 { small } else { (units - 1) * capacity + capacity / 2 };
    (0..length).map(|i| ((i * 13 + usize::from(seed) * 7 + 3) % 251) as u8).collect()
}

pub struct Built {
    pub name: String,
    pub width: HistoryDeviceWidth,
    /// 被判那一次发布之前的池（之前的写全持久）。
    pub base: MemoryPool,
    /// 被判那一次发布的录制流。
    pub publish_ops: Vec<RetainedOperation>,
    /// 已发布的带文件的版本（被判那一次也在内）。
    pub versions: Vec<PublishedVersion>,
    /// 被判那一次之前最新那一版（已确认）：恢复不许落到它之下。
    pub floor: Key,
    /// 被判那一次发布的 (实例, txg)。
    pub target: Key,
    pub data_unit_offsets: BTreeSet<u64>,
    /// 被判那一次之前最新那一版（floor）的根槽写与它那次发布的记录写落在哪：(盘, 偏移, 长度)。
    pub floor_root_location: (u32, u64, u64),
    pub floor_record_locations: Vec<(u32, u64, u64)>,
}

fn version_of(output: &TransactionOutput, content: &[u8]) -> PublishedVersion {
    PublishedVersion {
        instance: output.root.instance,
        checkpoint_txg: output.root.checkpoint_txg,
        content: content.to_vec(),
    }
}

fn key_of(output: &TransactionOutput) -> Key {
    (output.root.instance, output.root.checkpoint_txg)
}

fn data_offsets(output: &TransactionOutput) -> BTreeSet<u64> {
    output
        .rewritten
        .iter()
        .filter(|role| matches!(role, TransactionUnit::Data(_)))
        .map(|role| output.unit(*role).slot.to_device_offset().0)
        .collect()
}

fn mount(session: &mut Session) -> singlefs_core::mount::Mounted {
    mount_writable_with_space_admission(&session.parameters, &mut session.devices, SpaceAdmission::JudgedByTheFormula)
        .expect("可写挂载")
}

fn first_file(session: &mut Session, mounted: &mut singlefs_core::mount::Mounted, content: &[u8]) -> TransactionOutput {
    let PoolVersion::WithoutFile(current) = &mounted.current else { panic!("挂载之后还没有文件") };
    let root = current.root;
    let record_bytes = current.record_bytes.clone();
    let instance = mounted.output.instance;
    let mut writer = PoolWriter::new(&session.parameters, session.devices.as_mut_slice());
    publish_first_file(&mut writer, &mut mounted.allocator, &root, FirstFile { content, write_time_seconds: WRITE_TIME }, instance, &record_bytes)
        .expect("第一个文件")
}

/// 场景名：
/// - `first`：mkfs → 挂载 → 第一个文件（被判）
/// - `over`：mkfs → 挂载 → 第一个文件 → 覆盖写（被判，一条记录）
/// - `seq2` / `seq3`：mkfs → 挂载 → 第一个文件 → 顺序写 2 / 3 个数据单元（被判，2 / 3 条记录，共享内生块只在末条点名）
/// - `over_i2`：mkfs → 挂载 → 第一个文件 → 关掉重挂（实例 2）→ 覆盖写（被判）
/// - `rollback`：mkfs → 挂载 → 第一个文件 → 覆盖写 → 回退到第一个文件那一版（被判，挂着时的一次向前发布）
pub fn build(name: &str, width: HistoryDeviceWidth) -> Built {
    let mut session = Session::fresh(width);
    make_filesystem(&session.parameters, &mut session.devices).expect("mkfs");
    let mut mounted = mount(&mut session);
    let c1 = content(1, 1, 3000);
    let mut versions = Vec::new();
    let begin;
    let floor;
    let output;
    let target_content: Vec<u8>;
    match name {
        "first" => {
            floor = (mounted.current.root().instance, mounted.current.root().checkpoint_txg);
            begin = session.operation_count();
            output = first_file(&mut session, &mut mounted, &c1);
            target_content = c1.clone();
        }
        "over" | "seq2" | "seq3" | "over_i2" | "rollback" => {
            let f = first_file(&mut session, &mut mounted, &c1);
            versions.push(version_of(&f, &c1));
            let mut previous = f.clone();
            let mut instance = mounted.output.instance;
            if name == "over_i2" {
                drop(mounted);
                mounted = mount(&mut session);
                instance = mounted.output.instance;
                let PoolVersion::WithFile(current) = &mounted.current else { panic!("重挂之后应带文件") };
                previous = current.clone();
                versions.push(PublishedVersion { instance: previous.root.instance, checkpoint_txg: previous.root.checkpoint_txg, content: c1.clone() });
            }
            if name == "rollback" {
                let c2 = content(1, 2, 4000);
                let mut writer = PoolWriter::new(&session.parameters, session.devices.as_mut_slice());
                let p = publish_overwrite(&mut writer, &mut mounted.allocator, &previous, FirstFile { content: &c2, write_time_seconds: WRITE_TIME + 1 }, instance).expect("覆盖写");
                drop(writer);
                versions.push(version_of(&p, &c2));
                floor = key_of(&p);
                begin = session.operation_count();
                let mut current = p.clone();
                roll_back_by_a_forward_publish(&session.parameters, &mut session.devices, &mut mounted.allocator, &mut current, RollbackTarget { instance: f.root.instance, checkpoint_txg: f.root.checkpoint_txg }).expect("回退");
                output = current;
                target_content = c1.clone();
            } else {
                floor = key_of(&previous);
                begin = session.operation_count();
                let units = match name { "seq2" => 2, "seq3" => 3, _ => 1 };
                target_content = content(units, 2, 4000);
                let mut writer = PoolWriter::new(&session.parameters, session.devices.as_mut_slice());
                let file = FirstFile { content: &target_content, write_time_seconds: WRITE_TIME + 1 };
                output = if units == 1 {
                    publish_overwrite(&mut writer, &mut mounted.allocator, &previous, file, instance).expect("覆盖写")
                } else {
                    publish_sequential_write(&mut writer, &mut mounted.allocator, &previous, file, instance).expect("顺序写")
                };
            }
        }
        many if many.starts_with("overmany") => {
            // overmany<K>：第一个文件之后连着覆盖写 K 次（都整次落盘），被判的是第 K + 1 次覆盖写。根环转过之后按谓词回收，
            // 后面的发布会复用前面换下的槽：被判那一次的单元落在一个盘上已有旧单元（带可用单元头）的槽里。
            let k: usize = many["overmany".len()..].parse().expect("overmany<K>");
            let f = first_file(&mut session, &mut mounted, &c1);
            versions.push(version_of(&f, &c1));
            let instance = mounted.output.instance;
            let mut previous = f.clone();
            for round in 0..k {
                let c = content(1, (round % 200) as u8 + 10, 3000 + round);
                let mut writer = PoolWriter::new(&session.parameters, session.devices.as_mut_slice());
                match publish_overwrite(&mut writer, &mut mounted.allocator, &previous, FirstFile { content: &c, write_time_seconds: WRITE_TIME + 2 + round as u64 }, instance) {
                    Ok(next) => {
                        previous = next;
                        versions.push(version_of(&previous, &c));
                    }
                    Err(error) => panic!("第 {round} 次覆盖写被拒：{error:?}"),
                }
            }
            floor = key_of(&previous);
            begin = session.operation_count();
            target_content = content(1, 2, 4000);
            let mut writer = PoolWriter::new(&session.parameters, session.devices.as_mut_slice());
            output = publish_overwrite(&mut writer, &mut mounted.allocator, &previous, FirstFile { content: &target_content, write_time_seconds: WRITE_TIME + 1 }, instance).expect("覆盖写");
        }
        many if many.starts_with("floorreuse") => {
            // floorreuse<K>：第一个文件 → 覆盖写 K 次 → 抬 F 到现行那一版 → 覆盖写（被判）。抬 F 之后换下的槽可回收，
            // 被判那一次的单元落在盘上已有旧单元（带可用单元头）的槽里。
            let k: usize = many["floorreuse".len()..].parse().expect("floorreuse<K>");
            let f = first_file(&mut session, &mut mounted, &c1);
            versions.push(version_of(&f, &c1));
            let instance = mounted.output.instance;
            let mut previous = f.clone();
            for round in 0..k {
                let c = content(1, (round % 200) as u8 + 10, 3000 + round);
                let mut writer = PoolWriter::new(&session.parameters, session.devices.as_mut_slice());
                previous = publish_overwrite(&mut writer, &mut mounted.allocator, &previous, FirstFile { content: &c, write_time_seconds: WRITE_TIME + 2 + round as u64 }, instance).expect("覆盖写");
                versions.push(version_of(&previous, &c));
            }
            // 抬 F 的上限是现行 txg − 3（先试时 `RollbackFloorAboveCeiling` 报的 ceiling）。
            let floor_txg = singlefs_core::address::CheckpointTxg(previous.root.checkpoint_txg.0 - 3);
            let raised = singlefs_core::mount::raise_rollback_floor(&session.parameters, &mut session.devices, &mut mounted.allocator, &mut previous, floor_txg, singlefs_core::mount::ShadowLedger::On).expect("抬 F");
            let _ = raised;
            let last_content = versions.last().expect("有版本").content.clone();
            versions.push(version_of(&previous, &last_content));
            floor = key_of(&previous);
            begin = session.operation_count();
            target_content = content(1, 2, 4000);
            let mut writer = PoolWriter::new(&session.parameters, session.devices.as_mut_slice());
            output = publish_overwrite(&mut writer, &mut mounted.allocator, &previous, FirstFile { content: &target_content, write_time_seconds: WRITE_TIME + 1 }, instance).expect("覆盖写");
        }
        other => panic!("没有这个场景：{other}"),
    }
    let ops = session.operations();
    let mut base = MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(0), singlefs_core::address::DeviceIdentity(1)], width.device_bytes());
    base.apply(&ops[..begin]);
    versions.push(version_of(&output, &target_content));
    let _ = (InstanceGeneration(0), CheckpointTxg(0));
    let (prefix_writes, _) = crate::common::split(&ops[..begin], width);
    let root_positions: Vec<usize> = (0..prefix_writes.len()).filter(|w| prefix_writes[*w].kind == singlefs_harness::segments::StepKind::RootRecordFua).collect();
    let last_root = *root_positions.last().expect("被判那一次之前有根槽写");
    let previous_root = root_positions.iter().rev().nth(1).copied().unwrap_or(0);
    let location = |w: &singlefs_harness::memory_pool::RetainedWrite| (w.device.0, w.offset.0, w.length_in_bytes());
    let floor_root_location = location(&prefix_writes[last_root]);
    let floor_record_locations = (previous_root..last_root)
        .filter(|w| prefix_writes[*w].kind == singlefs_harness::segments::StepKind::JournalRecord)
        .map(|w| location(&prefix_writes[w]))
        .collect();
    Built {
        name: name.to_string(),
        width,
        base,
        publish_ops: ops[begin..].to_vec(),
        versions,
        floor,
        target: key_of(&output),
        data_unit_offsets: data_offsets(&output),
        floor_root_location,
        floor_record_locations,
    }
}
