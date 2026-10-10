//! 第 21 批 K3 探针（harness 档）：直接调 `witnessed_counter_holds_against_the_selected_version`（副本里改成 pub 的那一个），
//! 在手写的日志世界上核「判据对所选那一版单调」：版本 V1 比 V2 旧时，T(V2) 为真是不是总推出 T(V1) 为真。
//! 世界：4 次发布 P0..P3（每次 1 或 2 条记录，前两次实例 1、后两次实例 2，txg 递增），记录各自读得出 / 读不出，c_见证 取 0..=总数 + 1。
//! 同时把五步判法照 `mount_transparency.rs` 抄成纯函数 `model`，逐点与真函数比。
mod common;
use common::*;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::{
    record_offset, JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::choose_system_configuration;
use singlefs_core::root_record::UnmountMarker;
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::mount_transparency::witnessed_counter_holds_against_the_selected_version;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

#[derive(Clone, Copy)]
struct Rec { inst: u32, txg: u64, counter: u64, last: bool }

/// 五步判法的纯函数抄本（readable[k] 说第 k 条记录读不读得出）。
fn model(recs: &[Rec], readable: &[bool], c: u64, v: Option<(u32, u64)>) -> bool {
    if c == 0 { return false; }
    let Some(v) = v else { return false };
    let on_disk: Vec<&Rec> = recs.iter().zip(readable).filter(|(_, r)| **r).map(|(x, _)| x).collect();
    let last_of_v = on_disk.iter().filter(|r| (r.inst, r.txg) == v && r.last).map(|r| r.counter).max();
    if let Some(l) = last_of_v { return c > l; }
    let at_c = on_disk.iter().filter(|r| r.counter == c).map(|r| (r.inst, r.txg)).max();
    match at_c { Some(x) => x > v, None => true }
}

fn build_base(journal_bytes: u64) -> (MemoryPool, singlefs_core::system_configuration::SystemConfiguration, u64) {
    let stream = SharedStream::retaining_contents();
    let mut p = parameters();
    p.geometry.journal_ring_bytes = journal_bytes;
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32).map(|n| (DeviceIdentity(n),
        RecordingBlockDevice::with_shared_stream(DeviceIdentity(n), SparseBlockDevice::new(1 << 30, PhysicalBlockSizeInBytes(512)), stream.clone()))).collect();
    make_filesystem(&p, &mut devices).expect("mkfs");
    let pool = memory_pool_of_sparse_devices(&devices);
    let sc = choose_system_configuration(&pool).expect("sc");
    let fsid = unit_filesystem_identifier(&sc.immutable.filesystem_identifier);
    (pool, sc, fsid)
}

fn record_bytes(r: &Rec, ordinal: u32, fsid: u64) -> Vec<u8> {
    JournalRecord {
        instance: InstanceGeneration(r.inst), counter: r.counter, checkpoint_txg: CheckpointTxg(r.txg),
        transaction: 0, is_commit: true, ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal),
        place_in_publish: if r.last { JournalRecordPlaceInPublish::LastRecordOfThePublish } else { JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow },
        back_chain: 0, filesystem_identifier: fsid,
        new_tree_table: NodePointer::empty_root(), new_mapping_root: NodePointer::empty_root(),
        new_tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        new_rollback_floor: CheckpointTxg(0), new_instance_table: NodePointer::empty_root(),
        new_allocation_record_tree_root: NodePointer::empty_root(), new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence, named: Vec::new(),
    }.to_bytes()
}

#[test]
fn k3_monotone() {
    let journal_bytes: u64 = std::env::var("B21_JBYTES").ok().and_then(|v| v.parse().ok()).unwrap_or(4 << 20);
    let (base, sc, fsid) = build_base(journal_bytes);
    let mut mismatches_model_vs_code = 0u64;
    let mut calls = 0u64;
    let mut counterexamples_general: Vec<String> = Vec::new();
    let mut counterexamples_in_r: Vec<String> = Vec::new();
    let mut n_general_pairs = 0u64; let mut n_r_pairs = 0u64; let mut n_r0_pairs = 0u64; let mut r0_examples: Vec<String> = Vec::new(); let mut r0_counter = 0u64; let mut r_reverse = 0u64; let mut r_forward_true_true = 0u64;
    let mut reverse_false_to_true_general = 0u64; // T(V2) 假而 T(V1) 真（旧版更真）
    for shape in 0..16u32 {
        // 每次发布 1 或 2 条记录
        let n_of = |j: usize| 1 + ((shape >> j) & 1) as usize;
        let mut recs: Vec<Rec> = Vec::new(); let mut pub_of: Vec<usize> = Vec::new(); let mut ordinals: Vec<u32> = Vec::new();
        let mut counter = 0u64; let mut last_counter_of_pub = [0u64; 4];
        for j in 0..4usize {
            for k in 0..n_of(j) {
                counter += 1;
                recs.push(Rec { inst: if j < 2 { 1 } else { 2 }, txg: 10 + j as u64, counter, last: k + 1 == n_of(j) });
                pub_of.push(j); ordinals.push(u32::try_from(k).unwrap() + 1);
            }
            last_counter_of_pub[j] = counter;
        }
        let total = recs.len();
        for mask in 0..(1u32 << total) {
            let readable: Vec<bool> = (0..total).map(|k| (mask >> k) & 1 == 1).collect();
            let mut pool = base.clone();
            for (k, r) in recs.iter().enumerate() {
                if !readable[k] { continue; }
                let bytes = record_bytes(r, ordinals[k], fsid);
                let off = record_offset(r.counter, journal_bytes);
                for d in 0..2u32 { pool.devices.get_mut(&DeviceIdentity(d)).unwrap().write(off, &bytes); }
            }
            let vs: Vec<(u32, u64)> = (0..4).map(|j| (if j < 2 { 1 } else { 2 }, 10 + j as u64)).collect();
            for c in 0..=(total as u64 + 1) {
                let t_real: Vec<bool> = vs.iter().map(|v| witnessed_counter_holds_against_the_selected_version(&pool, &sc, c, Some((InstanceGeneration(v.0), CheckpointTxg(v.1)))).is_some()).collect();
                let t_model: Vec<bool> = vs.iter().map(|v| model(&recs, &readable, c, Some(*v))).collect();
                calls += 4;
                if t_real != t_model { mismatches_model_vs_code += 1; }
                for a in 0..4usize { for b in (a + 1)..4usize {
                    // 一般世界
                    n_general_pairs += 1;
                    if t_real[b] && !t_real[a] {
                        if counterexamples_general.len() < 6 { counterexamples_general.push(format!("shape={shape} mask={mask:0width$b} c={c} V1=P{a} V2=P{b}", width = total)); }
                    }
                    if !t_real[b] && t_real[a] { reverse_false_to_true_general += 1; }
                    // 现实域 R：早于 V1 的发布，记录全读得出；c 取某次发布的末条计数器（或 0），且不早于 P_{a-1} 的
                    let earlier_ok = (0..total).filter(|k| pub_of[*k] < a).all(|k| readable[k]);
                    let c_ok = c == 0 && a == 0 || (0..4usize).any(|j| last_counter_of_pub[j] == c && j + 1 >= a);
                    if earlier_ok { n_r0_pairs += 1; if t_real[b] && !t_real[a] { r0_counter += 1; if !c_ok && r0_examples.len() < 8 { r0_examples.push(format!("shape={shape} mask={mask:0width$b} c={c} V1=P{a} V2=P{b}", width = total)); } } }
                    if earlier_ok && c_ok {
                        if !t_real[b] && t_real[a] { r_reverse += 1; }
                        if t_real[b] && t_real[a] { r_forward_true_true += 1; }
                        n_r_pairs += 1;
                        if t_real[b] && !t_real[a] && counterexamples_in_r.len() < 6 { counterexamples_in_r.push(format!("shape={shape} mask={mask:0width$b} c={c} V1=P{a} V2=P{b}", width = total)); }
                    }
                }}
            }
        }
    }
    println!("K3 calls={calls} model_vs_code_mismatches={mismatches_model_vs_code}");
    println!("K3 general: pairs={n_general_pairs} counterexamples(T(V2) true, T(V1) false) listed={} first={:?}", counterexamples_general.len(), counterexamples_general);
    println!("K3 general: pairs with T(V2) false and T(V1) true = {reverse_false_to_true_general}");
    println!("K3 domain R0 (earlier publishes' records all readable, c arbitrary): pairs={n_r0_pairs} counterexamples={r0_counter} examples_outside_R={r0_examples:?}");
    println!("K3 domain R: pairs with T(V2) true and T(V1) true = {r_forward_true_true}; pairs with T(V2) false and T(V1) true = {r_reverse}");
    println!("K3 domain R: pairs={n_r_pairs} counterexamples listed={} first={:?}", counterexamples_in_r.len(), counterexamples_in_r);
}
