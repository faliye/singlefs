//! e171-conclusion-r2 攻方腿：照抄 e171-conclusion-r1 攻方 `opus_e171_history_probe.rs`，只删掉 unlink 一支与它的两个导入（E171 跑的那棵 s4 树没有 unlink 的入口），
//! 好在 E171 跑的那棵树上重做第 23 行「多建 233 个 inode」那一段。下面是原文件头。
//! e171-conclusion-r1 攻方腿的第二个探针（只在腿的副本上编、跑；不进 crates）：拿 E171 导出的一条流的全持久态（每条写都落了）当起点，
//! 在内存稀疏盘上接着走一小段用户历史，看三条 oracle 之外的那几样可观测量。用户动作的几步放开扫（rollback 的目标逐个试）。
//! 环境变量：OPUS_HIST_DUMP（流目录）、OPUS_HIST_CASE（inodes | remount_rollback | unlink）、OPUS_HIST_OUT（输出，追加）、OPUS_HIST_LABEL。
mod common;

use std::collections::BTreeMap;
use std::io::Write;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::transaction::{
    publish_new_inodes, PoolVersion, PoolWriter,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

fn rows(path: &std::path::Path) -> Vec<Vec<String>> {
    std::fs::read_to_string(path)
        .expect("读表")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split('\t').map(str::to_string).collect::<Vec<_>>())
        .filter(|c| c[0].chars().all(|ch| ch.is_ascii_digit()))
        .collect()
}

/// 全持久态：基镜像加每一次写（屏障不改镜像）。
fn final_image(dir: &std::path::Path) -> (u64, BTreeMap<u32, SparseDevice>) {
    let mut manifest = BTreeMap::new();
    for line in std::fs::read_to_string(dir.join("manifest.tsv")).expect("manifest").lines() {
        if let Some((k, v)) = line.split_once('\t') {
            manifest.insert(k.to_string(), v.to_string());
        }
    }
    let device_bytes: u64 = manifest["device_bytes"].parse().unwrap();
    let mut disks: BTreeMap<u32, SparseDevice> = manifest["devices"]
        .split(',')
        .map(|t| (t.trim().parse().unwrap(), SparseDevice::default()))
        .collect();
    let raw = std::fs::read(dir.join("base-sectors.bin")).unwrap();
    for r in raw.chunks(524) {
        let d = u32::from_le_bytes(r[0..4].try_into().unwrap());
        let s = u64::from_le_bytes(r[4..12].try_into().unwrap());
        disks.get_mut(&d).unwrap().write(DeviceOffsetInBytes(s * 512), &r[12..524]);
    }
    let payload = std::fs::read(dir.join("ops-payload.bin")).unwrap();
    for c in rows(&dir.join("ops.tsv")) {
        let device: u32 = c[1].parse().unwrap();
        let offset: u64 = c[3].parse().unwrap();
        let count: u64 = c[4].parse().unwrap();
        match c[2].as_str() {
            "barrier" => {}
            "write_zeroes" => disks.get_mut(&device).unwrap().zero_fill(DeviceOffsetInBytes(offset), count),
            _ => {
                let start: usize = c[5].parse().unwrap();
                disks.get_mut(&device).unwrap().write(DeviceOffsetInBytes(offset), &payload[start..start + count as usize]);
            }
        }
    }
    (device_bytes, disks)
}

fn reds(devices: &[(DeviceIdentity, SparseBlockDevice)], device_bytes: u64) -> String {
    let pool = MemoryPool {
        devices: devices.iter().map(|(d, b)| (*d, b.image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    };
    let red: Vec<String> = check_pool_image(&pool)
        .into_iter()
        .filter_map(|(n, v)| match v {
            InvariantVerdict::Violated(detail) => Some(format!("{n}:{}", detail.chars().take(80).collect::<String>())),
            _ => None,
        })
        .collect();
    let report = recover(&pool, JournalPolicy::Consult);
    let effective = report.effective_root.map_or("none".to_string(), |(i, t)| format!("{},{}", i.0, t.0));
    format!("recover={effective} o2_red={}", if red.is_empty() { "-".to_string() } else { red.join("|").replace(['\t', '\n', ' '], "_") })
}

#[test]
fn opus_probe_short_user_histories_from_an_exported_final_state() {
    let Ok(dir) = std::env::var("OPUS_HIST_DUMP") else {
        eprintln!("OPUS_HIST_DUMP 没给，什么都不做");
        return;
    };
    let case = std::env::var("OPUS_HIST_CASE").expect("OPUS_HIST_CASE");
    let label = std::env::var("OPUS_HIST_LABEL").unwrap_or_default();
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::var("OPUS_HIST_OUT").expect("OPUS_HIST_OUT"))
        .unwrap();
    let parameters = common::parameters();
    let (device_bytes, disks) = final_image(std::path::Path::new(&dir));
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = disks
        .into_iter()
        .map(|(d, s)| {
            let mut b = SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512));
            b.image = s;
            (DeviceIdentity(d), b)
        })
        .collect();
    writeln!(out, "{label}\t{case}\tstep=start\t{}", reds(&devices, device_bytes)).unwrap();
    let mut mounted = match mount_writable(&parameters, &mut devices) {
        Ok(m) => m,
        Err(e) => {
            writeln!(out, "{label}\t{case}\tstep=mount\terr {e:?}").unwrap();
            return;
        }
    };
    let instance = mounted.output.instance;
    {
        let root = mounted.current.root();
        writeln!(
            out,
            "{label}\t{case}\tstep=mount\tok instance={} new_root={},{} floor={} chosen={},{} effective={},{}\t{}",
            instance.0,
            root.instance.0,
            root.checkpoint_txg.0,
            root.rollback_floor.0,
            mounted.output.chosen_root.instance.0,
            mounted.output.chosen_root.checkpoint_txg.0,
            mounted.output.effective_root.instance.0,
            mounted.output.effective_root.checkpoint_txg.0,
            reds(&devices, device_bytes)
        )
        .unwrap();
    }
    let PoolVersion::WithFile(mut current) = mounted.current.clone() else {
        writeln!(out, "{label}\t{case}\tstep=current\tnot_a_version_with_file").unwrap();
        return;
    };
    match case.as_str() {
        "inodes" => {
            for (step, count) in [(1, 233_u64)] {
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                let result = publish_new_inodes(&mut writer, &mut mounted.allocator, &current, count, 120, instance);
                drop(writer);
                match result {
                    Ok(next) => {
                        let containers: Vec<String> = next
                            .inode_leaf_containers
                            .iter()
                            .map(|c| format!("{}:{}", c.contents.identity.container, c.contents.records.len()))
                            .collect();
                        writeln!(
                            out,
                            "{label}\t{case}\tstep=new_inodes_{step}\tok containers={}\t{}",
                            containers.join(","),
                            reds(&devices, device_bytes)
                        )
                        .unwrap();
                        current = next;
                    }
                    Err(e) => {
                        writeln!(out, "{label}\t{case}\tstep=new_inodes_{step}\terr {e:?}").unwrap();
                        return;
                    }
                }
            }
        }
        "remount_rollback" => {
            // 用户动作放开扫：回退目标取 (实例 0..=4, txg 1..=新根 txg) 每一个，每一个都从挂载之后那一刻的拷贝起。
            let newest = mounted.current.root().checkpoint_txg.0;
            let mut accepted = Vec::new();
            let mut tried = 0;
            for target_instance in 0..=4_u32 {
                for txg in 1..=newest {
                    tried += 1;
                    let mut trial_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = devices
                        .iter()
                        .map(|(d, b)| {
                            let mut c = SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512));
                            c.image = b.image.clone();
                            (*d, c)
                        })
                        .collect();
                    let mut allocator = mounted.allocator.clone();
                    let mut trial_current = current.clone();
                    let target = RollbackTarget {
                        instance: InstanceGeneration(target_instance),
                        checkpoint_txg: CheckpointTxg(txg),
                    };
                    if roll_back_by_a_forward_publish(&parameters, &mut trial_devices, &mut allocator, &mut trial_current, target).is_ok() {
                        accepted.push(format!("{target_instance},{txg}"));
                    }
                }
            }
            writeln!(out, "{label}\t{case}\tstep=rollback_sweep\ttried={tried} accepted={}", if accepted.is_empty() { "-".to_string() } else { accepted.join(";") }).unwrap();
        }
        other => panic!("不认得的 case {other}"),
    }
}
