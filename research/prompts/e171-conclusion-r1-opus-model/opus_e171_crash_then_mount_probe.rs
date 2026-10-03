//! e171-conclusion-r1 攻方腿的探针（只在腿的副本上编、跑；不进 crates）。
//! 读 E171 导出目录（中性格式：manifest / base-sectors.bin / ops.tsv / ops-payload.bin / torn.tsv / torn-payload.bin / states.tsv），
//! 逐态在内存稀疏盘上建崩溃镜像（与 O3 的 crash_image.py 同一叠法），核镜像摘要等于 states.tsv 记的，然后在同一份镜像的拷贝上：
//!   ① 实现的只读恢复（recover，看 journal）走到哪一版；② 池级 checker（O2）红了哪几条；③ 崩溃之后的一次可写挂载（mount_writable）的结局。
//! 环境变量：OPUS_PROBE_DUMPS（逗号分隔的 标签=流目录）、OPUS_PROBE_BATCHES（逗号分隔的批号，空=全部）、OPUS_PROBE_OUT（输出文件，追加）。
mod common;

use std::collections::BTreeMap;
use std::io::Write;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::sha256::sha256_hexadecimal;

struct Op {
    device: u32,
    kind: String,
    offset: u64,
    count: u64,
    payload: Option<Vec<u8>>,
}

struct Dump {
    device_bytes: u64,
    devices: Vec<u32>,
    base: Vec<(u32, u64, Vec<u8>)>,
    writes: Vec<Op>,
    torn: BTreeMap<usize, Vec<u8>>,
    states: Vec<(String, String, String, String, String, String)>,
}

fn rows(path: &std::path::Path) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(path).expect("读表");
    let mut out = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<String> = line.split('\t').map(str::to_string).collect();
        if out.is_empty() && !cols[0].trim_start_matches('-').chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        out.push(cols);
    }
    out
}

fn load(dir: &std::path::Path) -> Dump {
    let mut manifest = BTreeMap::new();
    for line in std::fs::read_to_string(dir.join("manifest.tsv")).expect("manifest").lines() {
        if let Some((k, v)) = line.split_once('\t') {
            manifest.insert(k.to_string(), v.to_string());
        }
    }
    let devices: Vec<u32> = manifest["devices"].split(',').map(|t| t.trim().parse().expect("盘号")).collect();
    let device_bytes: u64 = manifest["device_bytes"].parse().expect("盘长");
    let raw = std::fs::read(dir.join("base-sectors.bin")).expect("基镜像");
    assert_eq!(raw.len() % 524, 0);
    let base = raw
        .chunks(524)
        .map(|r| {
            (
                u32::from_le_bytes(r[0..4].try_into().unwrap()),
                u64::from_le_bytes(r[4..12].try_into().unwrap()),
                r[12..524].to_vec(),
            )
        })
        .collect();
    let payload = std::fs::read(dir.join("ops-payload.bin")).expect("写字节");
    let mut writes = Vec::new();
    for c in rows(&dir.join("ops.tsv")) {
        let kind = c[2].clone();
        if kind == "barrier" {
            continue;
        }
        let count: u64 = c[4].parse().expect("长度");
        let p = if c[5] == "-" {
            None
        } else {
            let s: usize = c[5].parse().expect("起点");
            Some(payload[s..s + count as usize].to_vec())
        };
        writes.push(Op { device: c[1].parse().unwrap(), kind, offset: c[3].parse().unwrap(), count, payload: p });
    }
    let torn_payload = std::fs::read(dir.join("torn-payload.bin")).expect("撕裂字节");
    let mut torn = BTreeMap::new();
    for c in rows(&dir.join("torn.tsv")) {
        let (o, s, l): (usize, usize, usize) = (c[0].parse().unwrap(), c[1].parse().unwrap(), c[2].parse().unwrap());
        torn.insert(o, torn_payload[s..s + l].to_vec());
    }
    let states = rows(&dir.join("states.tsv"))
        .into_iter()
        .map(|c| (c[0].clone(), c[1].clone(), c[2].clone(), c[3].clone(), c[4].clone(), c[5].clone()))
        .collect();
    Dump { device_bytes, devices, base, writes, torn, states }
}

fn image(dump: &Dump, landing: &str) -> BTreeMap<u32, SparseDevice> {
    let mut disks: BTreeMap<u32, SparseDevice> = dump.devices.iter().map(|d| (*d, SparseDevice::default())).collect();
    for (d, sector, bytes) in &dump.base {
        disks.get_mut(d).unwrap().write(DeviceOffsetInBytes(sector * 512), bytes);
    }
    for (ordinal, (op, landed)) in dump.writes.iter().zip(landing.chars()).enumerate() {
        let disk = disks.get_mut(&op.device).unwrap();
        match landed {
            '2' => match &op.payload {
                Some(p) => disk.write(DeviceOffsetInBytes(op.offset), p),
                None => {
                    assert_eq!(op.kind, "write_zeroes");
                    disk.zero_fill(DeviceOffsetInBytes(op.offset), op.count);
                }
            },
            '1' => disk.write(DeviceOffsetInBytes(op.offset), &dump.torn[&ordinal]),
            '0' => {}
            other => panic!("落地字符 {other}"),
        }
    }
    disks
}

fn digest(disks: &BTreeMap<u32, SparseDevice>) -> String {
    let mut message = Vec::new();
    for (d, disk) in disks {
        for (sector, bytes) in disk.written_sectors() {
            if bytes.iter().all(|b| *b == 0) {
                continue;
            }
            message.extend_from_slice(&d.to_le_bytes());
            message.extend_from_slice(&sector.to_le_bytes());
            message.extend_from_slice(bytes);
        }
    }
    sha256_hexadecimal(&message)
}

#[test]
fn opus_probe_crash_states_read_only_recovery_checker_and_writable_mount() {
    let Ok(spec) = std::env::var("OPUS_PROBE_DUMPS") else {
        eprintln!("OPUS_PROBE_DUMPS 没给，什么都不做");
        return;
    };
    let batches: Vec<String> = std::env::var("OPUS_PROBE_BATCHES")
        .unwrap_or_default()
        .split(',')
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect();
    let out_path = std::env::var("OPUS_PROBE_OUT").expect("OPUS_PROBE_OUT");
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(&out_path).expect("输出");
    let parameters = common::parameters();
    // 分片：OPUS_PROBE_SHARD=i/n 只判状态序号 mod n == i 的那些（几片并行跑，合起来一个不落）。
    let (shard, shards) = std::env::var("OPUS_PROBE_SHARD").ok().map_or((0, 1), |t| {
        let (a, b) = t.split_once('/').expect("i/n");
        (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap())
    });
    let skip_o2 = std::env::var("OPUS_PROBE_SKIP_O2").is_ok();
    let skip_recover = std::env::var("OPUS_PROBE_SKIP_RECOVER").is_ok();
    for item in spec.split(',') {
        let (label, dir) = item.split_once('=').expect("标签=目录");
        let dump = load(std::path::Path::new(dir));
        let mut judged = 0;
        for (state, layer0, segment, batch, landing, recorded) in &dump.states {
            if !batches.is_empty() && !batches.contains(batch) {
                continue;
            }
            if state.parse::<usize>().unwrap() % shards != shard {
                continue;
            }
            let started = std::time::Instant::now();
            judged += 1;
            let disks = image(&dump, landing);
            let digest_ok = digest(&disks) == *recorded;
            let pool = MemoryPool {
                devices: disks.iter().map(|(d, s)| (DeviceIdentity(*d), s.clone())).collect(),
                device_size_in_bytes: dump.device_bytes,
            };
            let effective = if skip_recover {
                "skipped".to_string()
            } else {
                recover(&pool, JournalPolicy::Consult)
                    .effective_root
                    .map_or("none".to_string(), |(i, t)| format!("{},{}", i.0, t.0))
            };
            let after_recover = started.elapsed().as_millis();
            let red: Vec<String> = if skip_o2 {
                vec!["skipped".to_string()]
            } else {
                check_pool_image(&pool)
                    .into_iter()
                    .filter_map(|(name, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then(|| name.to_string()))
                    .collect()
            };
            let after_o2 = started.elapsed().as_millis();
            let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = disks
                .into_iter()
                .map(|(d, s)| {
                    let mut device = SparseBlockDevice::new(dump.device_bytes, PhysicalBlockSizeInBytes(512));
                    device.image = s;
                    (DeviceIdentity(d), device)
                })
                .collect();
            let mounted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mount_writable(&parameters, &mut devices)));
            let mount_text = match mounted {
                Ok(Ok(m)) => {
                    let root = m.current.root();
                    format!("ok new_root={},{} floor={}", root.instance.0, root.checkpoint_txg.0, root.rollback_floor.0)
                }
                Ok(Err(error)) => {
                    let text = format!("{error:?}");
                    format!("err {}", text.chars().take(700).collect::<String>().replace(['\t', '\n', ' '], "_"))
                }
                Err(_) => "panic".to_string(),
            };
            writeln!(
                out,
                "{label}\t{state}\t{layer0}\t{segment}\t{batch}\tdigest_ok={digest_ok}\trecover={effective}\to2_red={}\tmount={mount_text}\tms={after_recover}/{after_o2}/{}",
                if red.is_empty() { "-".to_string() } else { red.join(",") },
                started.elapsed().as_millis()
            )
            .unwrap();
        }
        eprintln!("{label}: judged {judged}");
    }
}
