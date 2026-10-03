//! m3-ring-r2 攻方原型 G1：每一种发布「记录已持久、根槽没持久」时，恢复由记录施加出来的根（`MountOutput::effective_root`）
//! 与那次发布当初会写进根槽的根逐字段比；再把施加出来的根照 P1 写进 slot(h)，跑池级 checker；
//! 再把三个字段（实例表指针、分配记录树根、卸载记号）换成当初的，比字节是否相同。
//!
//! 用法：fill_bytes_by_kind <S>
#![allow(clippy::too_many_lines)]

use ring_window_proto::{
    build_prefix_without_file, content, devices_of, geometry, parameters, pool_of, publish_empty,
    recording_of, violated, Devices, FILESYSTEM_IDENTIFIER, IMAGE_BYTES, WRITE_TIME,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn fresh(slots_per_region: u64) -> Devices {
    let parameters = parameters(slots_per_region);
    let mut devices: Devices = (0..2u32)
        .map(|n| (DeviceIdentity(n), singlefs_harness::memory_pool::SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    devices
}

fn diff_fields(a: &RootRecord, b: &RootRecord) -> Vec<&'static str> {
    let mut out = Vec::new();
    if a.filesystem_identifier != b.filesystem_identifier { out.push("filesystem_identifier"); }
    if a.unmount_marker != b.unmount_marker { out.push("unmount_marker"); }
    if a.instance != b.instance { out.push("instance"); }
    if a.checkpoint_txg != b.checkpoint_txg { out.push("checkpoint_txg"); }
    if a.tree_table != b.tree_table { out.push("tree_table"); }
    if a.tree_identifier_watermark != b.tree_identifier_watermark { out.push("tree_identifier_watermark"); }
    if a.rollback_floor != b.rollback_floor { out.push("rollback_floor"); }
    if a.instance_table != b.instance_table { out.push("instance_table"); }
    if a.mapping_root != b.mapping_root { out.push("mapping_root"); }
    if a.allocation_record_tree_root != b.allocation_record_tree_root { out.push("allocation_record_tree_root"); }
    out
}

/// 对一条录下来的流：在第 `nth` 次根槽 FUA 写之前切（之前的写全落，之后一个不落），挂一次可写挂载（在拷贝上，只为拿读阶段交出的两条根），
/// 比施加出来的根与那次根槽写的字节，再把它补进 slot(h) 跑 checker。
fn judge(kind: &str, slots_per_region: u64, before: &MemoryPool, writes: &[RetainedWrite], nth: usize) {
    let parameters = parameters(slots_per_region);
    let root_writes: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::RootRecordFua).map(|(i, _)| i).collect();
    let Some(&cut) = root_writes.get(nth) else {
        println!("G1 kind={kind} nth={nth} no_such_root_write roots_in_stream={}", root_writes.len());
        return;
    };
    let true_bytes = writes[cut].bytes().expect("根槽写是普通写").to_vec();
    let true_root = RootRecord::parse_slot(&true_bytes, &FILESYSTEM_IDENTIFIER).expect("根槽写读得回");
    let mut crashed = before.clone();
    crashed.apply_writes(&writes[..cut]);
    let mut devices = devices_of(&crashed);
    let mounted = match mount_writable(&parameters, &mut devices) {
        Ok(m) => m,
        Err(error) => {
            println!("G1 kind={kind} nth={nth} txg={} next_mount_err={}", true_root.checkpoint_txg.0, format!("{error:?}").chars().take(100).collect::<String>());
            return;
        }
    };
    let chosen = mounted.output.chosen_root;
    let effective = mounted.output.effective_root;
    let applied = effective.checkpoint_txg > chosen.checkpoint_txg;
    let fields = diff_fields(&effective, &true_root);
    let slot_len = true_bytes.len();
    let rebuilt_bytes = effective.to_slot(slot_len);
    let mut corrected = effective;
    corrected.instance_table = true_root.instance_table;
    corrected.allocation_record_tree_root = true_root.allocation_record_tree_root;
    corrected.unmount_marker = true_root.unmount_marker;
    let corrected_bytes = corrected.to_slot(slot_len);
    // P1 补洞：在崩溃镜像上把施加出来的根写进 slot(h)（与那次根槽写同一个盘、同一个偏移），之后什么都不写，跑 checker。
    let fill = |bytes: &[u8]| {
        let mut image = crashed.clone();
        let mut write = writes[cut].clone();
        write.contents = singlefs_harness::memory_pool::WrittenContents::Bytes(bytes.to_vec());
        image.apply_writes(std::slice::from_ref(&write));
        image
    };
    let red_rebuilt = if applied { violated(&fill(&rebuilt_bytes)) } else { Vec::new() };
    let red_corrected = if applied { violated(&fill(&corrected_bytes)) } else { Vec::new() };
    let red_true = violated(&fill(&true_bytes));
    let red_crash = violated(&crashed);
    println!(
        "G1 kind={kind} nth={nth} txg={} chosen=({},{}) effective=({},{}) record_applied={applied} differing_fields={fields:?} rebuilt_bytes_equal={} three_fields_corrected_bytes_equal={} true_marker={:?} rebuilt_marker={:?} red_crash_state={red_crash:?} red_after_fill_rebuilt={red_rebuilt:?} red_after_fill_three_fields={red_corrected:?} red_after_fill_true_root={red_true:?}",
        true_root.checkpoint_txg.0, chosen.instance.0, chosen.checkpoint_txg.0, effective.instance.0, effective.checkpoint_txg.0,
        rebuilt_bytes == true_bytes, corrected_bytes == true_bytes, true_root.unmount_marker, effective.unmount_marker
    );
    let empty = singlefs_core::pointer::NodePointer::empty_root();
    println!(
        "G1 detail kind={kind} nth={nth} alloc_tree_root_empty: chosen={} effective={} true={} instance_table_same(chosen,true)={} true_floor={} effective_floor={}",
        chosen.allocation_record_tree_root == empty, effective.allocation_record_tree_root == empty, true_root.allocation_record_tree_root == empty,
        chosen.instance_table == true_root.instance_table, true_root.rollback_floor.0, effective.rollback_floor.0
    );
}

fn record<F: FnOnce(&mut Vec<(DeviceIdentity, singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>)>)>(
    slots_per_region: u64,
    before: &MemoryPool,
    body: F,
) -> Vec<RetainedWrite> {
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(before, &stream);
    if std::env::var("G1_UNMOUNT_ENTRY").is_ok_and(|v| v == "1") {
        stream.record_entry(singlefs_harness::RecordedPublishEntry::Unmount, || body(&mut recording));
    } else {
        body(&mut recording);
    }
    let operations = stream.retained_operations();
    singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries(&operations, &stream.entry_spans(), &geometry(&parameters(slots_per_region))).0
}

/// mkfs → 可写挂载 → 第一个文件 → 覆盖写 n 次：交回盘、分配器、现行版本、实例。
fn file_pool(slots_per_region: u64, overwrites: u64) -> (Devices, singlefs_core::allocator::PoolAllocator, TransactionOutput, singlefs_core::address::InstanceGeneration) {
    let parameters = parameters(slots_per_region);
    let mut devices = fresh(slots_per_region);
    let mounted = mount_writable(&parameters, &mut devices).expect("挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator;
    let first = content(0);
    let mut current = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(&mut pool, &mut allocator, mounted.current.root(), FirstFile { content: &first, write_time_seconds: WRITE_TIME }, instance, mounted.current.record_bytes()).expect("第一个文件")
    };
    for step in 0..overwrites {
        let bytes = content(step + 1);
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &bytes, write_time_seconds: WRITE_TIME + step + 1 }, instance).expect("覆盖写");
    }
    (devices, allocator, current, instance)
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let s: u64 = arguments[1].parse().expect("S");
    let parameters = parameters(s);
    // 1. 覆盖写。
    {
        let (devices, allocator, current, instance) = file_pool(s, 4);
        let before = pool_of(&devices);
        let writes = record(s, &before, |recording| {
            let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
            publish_overwrite(&mut pool, &mut allocator.clone(), &current, FirstFile { content: &content(77), write_time_seconds: WRITE_TIME + 77 }, instance).expect("覆盖写");
        });
        judge("overwrite", s, &before, &writes, 0);
    }
    // 2. 第一个文件（树表 0 条那一版之上）。
    {
        let mut devices = fresh(s);
        let mounted = mount_writable(&parameters, &mut devices).expect("挂载");
        let before = pool_of(&devices);
        let writes = record(s, &before, |recording| {
            let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
            publish_first_file(&mut pool, &mut mounted.allocator.clone(), mounted.current.root(), FirstFile { content: &content(5), write_time_seconds: WRITE_TIME }, mounted.output.instance, mounted.current.record_bytes()).expect("第一个文件");
        });
        judge("first_file", s, &before, &writes, 0);
    }
    // 2b. 第一个文件，树表 0 条那一版之上、池上已有几次可写挂载（分配记录树根非空）。
    for mounts in [2u64, 3, 4] {
        let mut devices = fresh(s);
        for _ in 1..mounts {
            mount_writable(&parameters, &mut devices).expect("前面几次挂载");
        }
        let mounted = mount_writable(&parameters, &mut devices).expect("挂载");
        let before = pool_of(&devices);
        let writes = record(s, &before, |recording| {
            let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
            publish_first_file(&mut pool, &mut mounted.allocator.clone(), mounted.current.root(), FirstFile { content: &content(5), write_time_seconds: WRITE_TIME }, mounted.output.instance, mounted.current.record_bytes()).expect("第一个文件");
        });
        judge(&format!("first_file_after_{mounts}_mounts"), s, &before, &writes, 0);
    }
    // 3. 带文件那一版上的空发布（与暖机同一个做法）。
    {
        let (devices, allocator, current, instance) = file_pool(s, 4);
        let before = pool_of(&devices);
        let writes = record(s, &before, |recording| {
            let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
            publish_empty(&mut pool, &mut allocator.clone(), &current, instance).expect("空发布");
        });
        judge("empty_on_file_version", s, &before, &writes, 0);
    }
    // 4. 准入抬 F（一串空发布带新 F）。
    {
        let (devices, allocator, current, _) = file_pool(s, 8);
        let before = pool_of(&devices);
        let mut done = None;
        for back in [4u64, 5, 6, 3, 2, 1] {
            let target = CheckpointTxg(current.root.checkpoint_txg.0.saturating_sub(back));
            let mut probe_devices = devices_of(&before);
            if raise_rollback_floor(&parameters, &mut probe_devices, &mut allocator.clone(), &mut current.clone(), target, ShadowLedger::On).is_ok() {
                done = Some(target);
                break;
            }
        }
        let target = done.expect("有一个 F 抬得上去");
        let writes = record(s, &before, |recording| {
            let mut devices: Vec<_> = std::mem::take(recording);
            raise_rollback_floor(&parameters, &mut devices, &mut allocator.clone(), &mut current.clone(), target, ShadowLedger::On).expect("抬 F");
            *recording = devices;
        });
        println!("G1 note kind=raise_floor new_floor={}", target.0);
        for nth in 0..3 {
            judge("raise_floor", s, &before, &writes, nth);
        }
    }
    // 5. 正常卸载（推 2 或 3 次带记号的空发布）。两个起点：现行 txg 除以 3 余 0/2 与余 1。
    for overwrites in [4u64, 5, 6] {
        let (devices, allocator, current, _) = file_pool(s, overwrites);
        let before = pool_of(&devices);
        std::env::set_var("G1_UNMOUNT_ENTRY", "1");
        let writes = record(s, &before, |recording| {
            let mut devices: Vec<_> = std::mem::take(recording);
            unmount(&parameters, &mut devices, &mut allocator.clone(), &mut PoolVersion::WithFile(current.clone()), ShadowLedger::On).expect("卸载");
            *recording = devices;
        });
        std::env::remove_var("G1_UNMOUNT_ENTRY");
        println!("G1 note kind=unmount current_txg={}", current.root.checkpoint_txg.0);
        for nth in 0..3 {
            judge(&format!("unmount_from_txg{}", current.root.checkpoint_txg.0), s, &before, &writes, nth);
        }
    }
    // 6. 管理员回退（向前发一次 cur + 1，内容是目标那一版）。
    {
        let (devices, allocator, current, _) = file_pool(s, 6);
        let before = pool_of(&devices);
        let target = RollbackTarget { instance: current.root.instance, checkpoint_txg: CheckpointTxg(current.root.checkpoint_txg.0 - 3) };
        let writes = record(s, &before, |recording| {
            let mut devices: Vec<_> = std::mem::take(recording);
            roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator.clone(), &mut current.clone(), target).expect("回退");
            *recording = devices;
        });
        judge("admin_rollback", s, &before, &writes, 0);
    }
    // 7. 写行（带文件那一版之上的新实例）：可写挂载本身。
    {
        let (devices, _, _, _) = file_pool(s, 4);
        let before = pool_of(&devices);
        let writes = record(s, &before, |recording| {
            mount_writable(&parameters, recording).expect("挂载");
        });
        judge("row_on_file_version", s, &before, &writes, 0);
        judge("first_warm_up_on_file_version", s, &before, &writes, 1);
    }
    // 8. 写行（树表 0 条那一版之上）与它之后的暖机。
    {
        let mut devices = fresh(s);
        for _ in 0..3 {
            mount_writable(&parameters, &mut devices).expect("挂载");
        }
        let before = pool_of(&devices);
        let writes = record(s, &before, |recording| {
            mount_writable(&parameters, recording).expect("挂载");
        });
        judge("row_on_version_without_file", s, &before, &writes, 0);
        judge("first_warm_up_on_version_without_file", s, &before, &writes, 1);
    }
    let _ = build_prefix_without_file;
}
