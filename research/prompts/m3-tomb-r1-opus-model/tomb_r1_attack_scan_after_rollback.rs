//! m3-tomb-r1 攻方腿（副本上的原型，被攻过零轮）：管理员回退之后，按 C113 定案 P2 / P3 的字面
//! （已发布谓词滤掉未发布的、同身份取择新键最大者）在盘上扫码 3 与码 1，看扫出来的「现行版本」是不是回退那次发布引用的那一版。
//! 盘是两块内存稀疏盘（不建镜像文件）。历史：mkfs → 取号 1 → 暖机 → A（新池新建文件）→ B、C（覆盖写）→ 挂着回退到 A（D）。
//! 只读 core 的公开函数；扫描与择新是这个文件自己按条款写的，不是 crates 里的实现（crates 里没有扫描重建）。

mod common;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::ImageReader;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{is_a_user_visible_unit, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unit::{parse_data_unit, parse_packed_unit, PACKED_TYPE_INODE};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::segments::{closed_form_state_count, segment_sizes_text, split_into_segments, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;

type Dev = RecordingBlockDevice<SparseBlockDevice>;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length).map(|i| u8::try_from((i * 7 + seed) % 253).expect("<256")).collect()
}

fn overwrite(devices: &mut Vec<(DeviceIdentity, Dev)>, allocator: &mut PoolAllocator, previous: &TransactionOutput, content: &[u8]) -> TransactionOutput {
    let p = parameters();
    let mut writer = PoolWriter::new(&p, devices.as_mut_slice());
    publish_overwrite(&mut writer, allocator, previous, FirstFile { content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, InstanceGeneration(1)).expect("覆盖写")
}

fn memory_pool(devices: &[(DeviceIdentity, Dev)]) -> singlefs_harness::memory_pool::MemoryPool {
    singlefs_harness::memory_pool::MemoryPool {
        devices: devices.iter().map(|(i, d)| (*i, d.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 一片 inode 叶里 inode 1 那条记录的 size（偏移 40）与 nlink（偏移 36）。
fn inode_one(records: &[Vec<u8>]) -> Option<(u64, u32)> {
    records.iter().find(|r| u64::from_le_bytes(r[0..8].try_into().unwrap()) == 1).map(|r| {
        (u64::from_le_bytes(r[40..48].try_into().unwrap()), u32::from_le_bytes(r[36..40].try_into().unwrap()))
    })
}

#[test]
fn scan_after_a_mounted_rollback_picks_the_abandoned_branch() {
    let p = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, Dev)> = (0..2u32)
        .map(|n| (DeviceIdentity(n), RecordingBlockDevice::with_shared_stream(DeviceIdentity(n), SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)), stream.clone())))
        .collect();
    let genesis = make_filesystem(&p, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES), DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES)]);
    allocator.mark_format_time_units(Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }, Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 });
    let a = {
        let mut writer = PoolWriter::new(&p, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(&mut writer, &mut allocator, warm.roots.last().expect("根"), FirstFile { content: &file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS }, instance, &warm.last_record_bytes).expect("A")
    };
    let before_b = stream.operations().len();
    let b = overwrite(&mut devices, &mut allocator, &a, &content_of(4100, 3));
    let before_c = stream.operations().len();
    let c = overwrite(&mut devices, &mut allocator, &b, &content_of(2500, 11));
    let before_d = stream.operations().len();
    let mut current = c.clone();
    roll_back_by_a_forward_publish(&p, &mut devices, &mut allocator, &mut current, RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: a.root.checkpoint_txg }).expect("回退到 A");
    let d = current;
    let operations = stream.operations();
    for (name, slice) in [("overwrite_B", &operations[before_b..before_c]), ("overwrite_C", &operations[before_c..before_d]), ("rollback_D", &operations[before_d..])] {
        let mut segments = split_into_segments(slice, &common::geometry());
        let base = closed_form_state_count(&segments);
        let sizes = segment_sizes_text(&segments);
        // 推的：候选多出来的单元写（每个单元两份）放进主段（最长的那一段：单元都在第一道屏障之前写）
        let main = (0..segments.len()).max_by_key(|i| segments[*i].iter().filter(|k| **k != StepKind::Barrier).count()).expect("至少一段");
        let mut extra_cells = Vec::new();
        for extra in [2usize, 4, 6] {
            let mut grown = segments.clone();
            for _ in 0..extra { grown[main].push(StepKind::UnitWrite); }
            extra_cells.push(format!("plus{extra}={}", closed_form_state_count(&grown)));
        }
        segments.clear();
        println!("E name=segments publish={name} measured_sizes={sizes} measured_two_state_closed_form={base} inferred_with_extra_unit_writes_in_main_segment:{}", extra_cells.join(","));
    }
    let mount_root_txg = d.root.checkpoint_txg.0;
    let referenced: Vec<(String, u64)> = d.units.iter().filter(|u| is_a_user_visible_unit(u.identity)).map(|u| (format!("{:?}", u.identity), u.slot.0)).collect();
    println!("E name=history txgs=A{}/B{}/C{}/D{} instance_of_d={} d_user_visible_units={:?}", a.root.checkpoint_txg.0, b.root.checkpoint_txg.0, c.root.checkpoint_txg.0, mount_root_txg, d.root.instance.0, referenced);
    let image = memory_pool(&devices);
    // 扫：每块盘每个候选槽读 32768 试码 3 与码 1。只有一个实例（1）、没有实例表行 ⇒ 已发布谓词 = 诞生代号 ≤ 挂载根 txg（D18 已定项 11「i == i_now」那一支）。
    let mut leaf_versions: BTreeMap<(u64, u64, u64), Vec<(u64, u32, u64, u64, Option<(u64, u32)>)>> = BTreeMap::new();
    let mut data_versions: BTreeMap<(u64, u64, u64), Vec<(u64, u32, u64, u64, u16)>> = BTreeMap::new();
    for device in image.devices() {
        for slot in image.candidate_unit_slots(device).unwrap_or_default() {
            let Some(bytes) = image.read(device, slot * 16384, 32768) else { continue };
            if let Ok(h) = parse_packed_unit(&bytes) {
                if h.identity.record_type == PACKED_TYPE_INODE && h.birth_txg.0 <= mount_root_txg {
                    let key = (h.identity.birth_tree.0, h.identity.container, h.identity.container_birth.0);
                    let v = (h.birth_txg.0, h.write_order.instance.0, h.write_order.transaction, slot, inode_one(&h.records));
                    let e = leaf_versions.entry(key).or_default();
                    if !e.iter().any(|x| x.0 == v.0 && x.2 == v.2) { e.push(v); }
                }
            } else if let Ok(h) = parse_data_unit(&bytes) {
                if h.birth_txg.0 <= mount_root_txg {
                    let key = (h.identity.object, h.identity.object_birth.0, h.identity.anchor_offset);
                    let v = (h.birth_txg.0, h.write_order.instance.0, h.write_order.transaction, slot, h.declared_length);
                    let e = data_versions.entry(key).or_default();
                    if !e.iter().any(|x| x.0 == v.0 && x.2 == v.2) { e.push(v); }
                }
            }
        }
    }
    let referenced_slots: Vec<u64> = d.units.iter().filter(|u| is_a_user_visible_unit(u.identity)).map(|u| u.slot.0).collect();
    for (key, versions) in &mut leaf_versions {
        versions.sort();
        let newest = versions.last().expect("至少一版");
        println!("E name=inode_leaf_versions identity(tree,container,birth)={key:?} published_versions(birth,inst,txn,slot,inode1(size,nlink))={versions:?} p3_newest_birth={} p3_newest_is_referenced_by_d={} d_references_birth={:?}", newest.0, referenced_slots.contains(&newest.3), versions.iter().find(|v| referenced_slots.contains(&v.3)).map(|v| v.0));
    }
    for (key, versions) in &mut data_versions {
        versions.sort();
        let newest = versions.last().expect("至少一版");
        println!("E name=data_unit_versions key(object,object_birth,anchor)={key:?} published_versions(birth,inst,txn,slot,declared_len)={versions:?} p3_newest_birth={} p3_newest_is_referenced_by_d={}", newest.0, referenced_slots.contains(&newest.3));
    }
    let _ = CheckpointTxg(0);
}

