//! 攻方探针（副本，不入库）：贴边写满之后 unlink 一个空 inode，读准入读数（可用、defer 里的槽、空槽、标签 6）。
#![allow(dead_code, clippy::all)]
mod common_admission;

use common_admission::*;
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::{admission_reading_before_a_publish, committed_reservation_recorded_in};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;

fn clone_pool(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    let s = pool.session();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: pool
            .devices
            .iter()
            .map(|(id, d)| {
                let mut n = SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512));
                n.image = d.image.clone();
                (*id, n)
            })
            .collect(),
        session: Some(MountedSession {
            allocator: s.allocator.clone(),
            current: s.current.clone(),
            instance: s.instance,
            shadow_ledger: s.shadow_ledger,
            parameters_and_device_table: s.parameters_and_device_table.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

/// (可用槽 min over devices, 可用+defer 槽 min, 已分配 dev0, defer dev0, 标签6 字节, 退役记录条数, 墓碑容器数, 退役容器数)
fn reading(pool: &PoolUnderTest<SparseBlockDevice>) -> String {
    let s = pool.session();
    let v = pool.current_file_version();
    let r = admission_reading_before_a_publish(&s.allocator, Some(v));
    let av: Vec<i128> = r.available_on_each_device().iter().map(|(_, a)| a.0).collect();
    let deferred: Vec<u64> = s.allocator.devices.iter().map(|d| d.deferred_slots()).collect();
    let allocated: Vec<u64> = s.allocator.devices.iter().map(|d| d.allocated_slots()).collect();
    let free: Vec<u64> = s.allocator.devices.iter().map(|d| d.free_slots()).collect();
    let slot = i128::from(SLOT_BYTES);
    let avail_slots: Vec<String> = av.iter().map(|a| format!("{:.3}", (*a as f64) / (slot as f64))).collect();
    let settled: Vec<String> = av
        .iter()
        .zip(&deferred)
        .map(|(a, d)| format!("{:.3}", ((*a + i128::from(*d) * slot) as f64) / (slot as f64)))
        .collect();
    let retire: usize = v.retirement_containers.iter().map(|c| c.contents.records.len()).sum();
    format!(
        "avail={:?} settled={:?} alloc={:?} deferred={:?} free={:?} tag6={} retire_records={} tomb_containers={} retire_containers={} leaves={:?}",
        avail_slots,
        settled,
        allocated,
        deferred,
        free,
        committed_reservation_recorded_in(Some(v)).0,
        retire,
        v.tombstone_containers.len(),
        v.retirement_containers.len(),
        v.inode_leaf_containers.iter().map(|c| c.contents.records.len()).collect::<Vec<_>>()
    )
}

fn outcome<T>(r: &Result<T, UserChangeRefused>) -> String {
    match r {
        Ok(_) => "ok".into(),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => "refused:NoSpaceAfterRaisingTheFloor".into(),
        Err(e) => format!("refused:{}", format!("{e:?}").chars().take(80).collect::<String>()),
    }
}

fn base(new_inodes: u64, pre_unlinks: &[u64]) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.write_time_seconds += 1;
    let w = pool.write_time_seconds;
    pool.publish(UserChange::NewInodes { count: new_inodes, write_time_seconds: w }, None)
        .expect("建 inode");
    for &n in pre_unlinks {
        pool.write_time_seconds += 1;
        let w = pool.write_time_seconds;
        pool.publish(UserChange::Unlink { inode: InodeNumber(n), write_time_seconds: w }, None)
            .expect("前置 unlink");
    }
    pool
}

fn sweep(name: &str, new_inodes: u64, pre_unlinks: &[u64], victim: u64) {
    let base_pool = base(new_inodes, pre_unlinks);
    println!("BASE {name} {}", reading(&base_pool));
    let cap = data_unit_payload_capacity();
    let mut last_ok = None;
    let du_values: Vec<usize> = std::env::var("PROBE_DU")
        .ok()
        .map(|s| s.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or_else(|| (1..=140).collect());
    for du in du_values {
        let mut pool = clone_pool(&base_pool);
        let w = pool.sequential_write(du * cap - cap / 2);
        if w.is_err() {
            println!("ROW {name} du={du} write={}", outcome(&w));
            continue;
        }
        last_ok = Some(du);
        let before = reading(&pool);
        pool.write_time_seconds += 1;
        let wt = pool.write_time_seconds;
        let u = pool.publish(UserChange::Unlink { inode: InodeNumber(victim), write_time_seconds: wt }, None);
        let raises = match &u { Ok(p) => p.floor_raises.len(), Err(_) => 0 };
        let after = if u.is_ok() { reading(&pool) } else { String::from("-") };
        // 删完之后再推根环（空发布，放回 defer）
        let pushed = pool
            .session
            .as_mut()
            .unwrap()
            .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(&mut pool.devices)
            .map(|_| "ok")
            .unwrap_or("refused");
        let settled = reading(&pool);
        // 之后再建一个空 inode（写入类发布，判式子）
        let c = pool.create_one_inode();
        println!(
            "ROW {name} du={du} write=ok unlink={} raises={raises} push={pushed} create_after={}\n  BEFORE {before}\n  AFTER  {after}\n  PUSHED {settled}",
            outcome(&u),
            outcome(&c)
        );
    }
    println!("LAST_OK {name} {last_ok:?}");
}

#[test]
fn probe_a_retire_opens_tombstone_already_open() {
    sweep("A", 234, &[2], 3);
}
#[test]
fn probe_b_control_no_absorb_tombstone_already_open() {
    sweep("B", 235, &[2], 3);
}
#[test]
fn probe_c_first_unlink_absorbs() {
    sweep("C", 233, &[], 2);
}
#[test]
fn probe_d_first_unlink_no_absorb() {
    sweep("D", 234, &[], 2);
}
