//! e171-conclusion-r2 攻方腿的探针（只在腿的副本上编、跑；不进 crates）。
//! 读中性格式的导出目录（与 e171-conclusion-r1 攻方的 `opus_e171_crash_then_mount_probe.rs` 同一个读法，那一段照抄），逐态：
//!   ① 实现的只读恢复走到哪一版、读回什么；② O2（池级 checker）红了哪几条；
//!   ③ O1-崩溃判官：调 `singlefs_harness::model::crash_recovery_disagreement`，模型提交过的版本由 OPUS_JUDGE_COMMITTED 给
//!      （`i,t:none` 或 `i,t:<内容名>`，分号隔开），盘上已持久的最新根逐态由 `<导出目录>/judge-extra.tsv` 给（态号 \t 实例,txg）；
//!   ④ RefFS：调 `singlefs_reference_filesystem`，操作序列由 OPUS_JUDGE_REFFS 给（照 harness 翻译层第 13 行的对应：
//!      发布第一个文件 = 建第一个文件 + 发布调用；覆盖写 = 覆盖 + 发布调用；逗号隔开 `create:<内容名>`、`overwrite:<内容名>`、`publish`），
//!      崩溃落在最后一个操作里（hi = 操作数）；
//!   ⑤ 崩溃之后可写挂载两次：第一次的结局、第一次之后盘上逐字节变没变、第二次的结局；第一次被拒时另交崩溃注入第二截的
//!      `judge_writable_mount_refusal` 判拒得合不合法。
//! 环境变量：OPUS_JUDGE_DUMPS（标签=目录，逗号分隔）、OPUS_JUDGE_OUT（追加）、OPUS_JUDGE_BATCHES（只判这几批，空=全部）、
//! OPUS_JUDGE_SHARD（i/n：只判态号 mod n == i 的那些）、OPUS_JUDGE_SKIP_MOUNT（不做⑤）。
mod common;

use std::collections::BTreeMap;
use std::io::Write;
use std::rc::Rc;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::model::{crash_recovery_disagreement, ModelRootKey};
use singlefs_harness::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal, DiskAfterTheRefusal, ReadOnlyReadBackOfTheCrashImage, WritableMountRefusalVerdict,
};
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_reference_filesystem::{
    FileContent, Operation, OperationNumber, OperationOutcome, ReferenceFileSystem, VisibleState,
};

fn named_content(name: &str) -> Vec<u8> {
    match name {
        "first" => (0..3000usize).map(|i| u8::try_from(i % 251).unwrap()).collect(),
        "second" => (0..4100usize).map(|i| u8::try_from((i * 7 + 3) % 253).unwrap()).collect(),
        other => panic!("内容名 {other}"),
    }
}

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

fn key(text: &str) -> ModelRootKey {
    let (i, t) = text.split_once(',').expect("i,t");
    model_root_key(InstanceGeneration(i.parse().unwrap()), CheckpointTxg(t.parse().unwrap()))
}

fn reffs_vote(script: &str, recovered: &VisibleState) -> String {
    let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
    let mut count = 0;
    for step in script.split(',') {
        let operation = match step.split_once(':') {
            Some(("create", c)) => Operation::CreateTheFirstFile(FileContent(named_content(c))),
            Some(("overwrite", c)) => Operation::OverwriteTheFile(FileContent(named_content(c))),
            None if step == "publish" => Operation::ExplicitPublish,
            _ => panic!("RefFS 步 {step}"),
        };
        model.submit(operation, OperationOutcome::Succeeded).expect("RefFS 收这一步");
        count += 1;
    }
    match model.allowed_set_for_a_crash_in(OperationNumber(count)) {
        Ok(allowed) if allowed.contains(recovered) => format!("green(lo={},hi={})", allowed.lower_bound.0, allowed.upper_bound.0),
        Ok(allowed) => format!("red(lo={},hi={},allowed={})", allowed.lower_bound.0, allowed.upper_bound.0, allowed.allowed.len()),
        Err(e) => format!("not_judged:{e:?}"),
    }
}

/// 第三样交回崩溃注入第二截那一道「拒得合不合法」（`judge_writable_mount_refusal`）在这一次拒上的判定：拒的判据在崩溃镜像上复算、
/// 盘上变没变照实交、只读读出交「在允许集合里」（这几态 O1-崩溃判官判绿，E171 产物里 242、269 两行崩溃判官 0 态红）。
fn mount_once(
    dump: &Dump,
    disks: &BTreeMap<u32, SparseDevice>,
    crash_image: &MemoryPool,
    selected: Option<(InstanceGeneration, CheckpointTxg)>,
) -> (String, String, String) {
    let parameters = common::parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = disks
        .iter()
        .map(|(d, s)| {
            let mut device = SparseBlockDevice::new(dump.device_bytes, PhysicalBlockSizeInBytes(512));
            device.image = s.clone();
            (DeviceIdentity(*d), device)
        })
        .collect();
    let mounted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mount_writable(&parameters, &mut devices)));
    let after: BTreeMap<u32, SparseDevice> = devices.iter().map(|(d, dev)| (d.0, dev.image.clone())).collect();
    let unchanged = digest(&after) == digest(disks);
    let mut refusal_verdict = "-".to_string();
    let text = match mounted {
        Ok(Ok(_)) => "ok".to_string(),
        Ok(Err(error)) => {
            let disk = if unchanged { DiskAfterTheRefusal::ByteForByteUnchanged } else { DiskAfterTheRefusal::Changed };
            refusal_verdict = match judge_writable_mount_refusal(crash_image, selected, &error, disk, ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet) {
                WritableMountRefusalVerdict::Legal(r) => format!("legal:{r:?}"),
                WritableMountRefusalVerdict::Illegal { reason, conditions_not_met } => format!("illegal:{reason:?}:{conditions_not_met:?}").replace(' ', ""),
                WritableMountRefusalVerdict::ReasonNotOnTheD23List => "reason_not_on_the_d23_list".to_string(),
            };
            let t = format!("{error:?}");
            let reason = ["AgainstTheRecordAtTheWitnessedCounter", "AgainstTheSelectedVersionsLastRecord", "SomeSystemConfigurationSlotUnreadOrUnverified", "Undecidable"]
                .iter()
                .find(|r| t.contains(*r))
                .map_or_else(|| t.chars().take(60).collect::<String>().replace(['\t', '\n', ' '], "_"), |r| (*r).to_string());
            format!("refused:{reason}")
        }
        Err(_) => "panic".to_string(),
    };
    (text, digest(&after), refusal_verdict)
}

#[test]
fn opus_r2_judge_states_with_o1_crash_judge_o2_reffs_and_two_writable_mounts() {
    let Ok(spec) = std::env::var("OPUS_JUDGE_DUMPS") else {
        eprintln!("OPUS_JUDGE_DUMPS 没给，什么都不做");
        return;
    };
    let out_path = std::env::var("OPUS_JUDGE_OUT").expect("OPUS_JUDGE_OUT");
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(&out_path).expect("输出");
    let committed: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = std::env::var("OPUS_JUDGE_COMMITTED")
        .expect("OPUS_JUDGE_COMMITTED")
        .split(';')
        .map(|item| {
            let (k, c) = item.split_once(':').expect("i,t:内容");
            (key(k), (c != "none").then(|| Rc::from(named_content(c).into_boxed_slice())))
        })
        .collect();
    let reffs_script = std::env::var("OPUS_JUDGE_REFFS").expect("OPUS_JUDGE_REFFS");
    let do_mount = std::env::var("OPUS_JUDGE_SKIP_MOUNT").is_err();
    for item in spec.split(',') {
        let (label, dir) = item.split_once('=').expect("标签=目录");
        let dump = load(std::path::Path::new(dir));
        // judge-extra.tsv 没有（E171 自己的导出目录只读，不往里写）时盘上已持久的最新根记「-」、O1-崩溃判官按没有这一条判据算
        let persisted: BTreeMap<String, String> = std::fs::read_to_string(std::path::Path::new(dir).join("judge-extra.tsv"))
            .unwrap_or_default()
            .lines()
            .map(|l| {
                let (s, p) = l.split_once('\t').expect("态\t实例,txg");
                (s.to_string(), p.to_string())
            })
            .collect();
        let batches: Vec<String> = std::env::var("OPUS_JUDGE_BATCHES").unwrap_or_default().split(',').filter(|t| !t.is_empty()).map(str::to_string).collect();
        let (shard, shards) = std::env::var("OPUS_JUDGE_SHARD").ok().map_or((0, 1), |t| {
            let (a, b) = t.split_once('/').expect("i/n");
            (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap())
        });
        for (state, _layer0, _segment, batch, landing, recorded) in &dump.states {
            if !batches.is_empty() && !batches.contains(batch) {
                continue;
            }
            if state.parse::<usize>().unwrap() % shards != shard {
                continue;
            }
            let disks = image(&dump, landing);
            let digest_ok = digest(&disks) == *recorded;
            let pool = MemoryPool {
                devices: disks.iter().map(|(d, s)| (DeviceIdentity(*d), s.clone())).collect(),
                device_size_in_bytes: dump.device_bytes,
            };
            let report = recover(&pool, JournalPolicy::Consult);
            let effective = report.effective_root.map_or("none".to_string(), |(i, t)| format!("{},{}", i.0, t.0));
            let visible = match &report.outcome {
                RecoveryOutcome::NoFile { .. } => Some(VisibleState::NoFile),
                RecoveryOutcome::FileRead { content, .. } => Some(VisibleState::OneFile(FileContent(content.to_vec()))),
                RecoveryOutcome::Failed { .. } => None,
            };
            let read_back = observed_read_back_after_a_crash(&report);
            let persisted_root = persisted.get(state).map(|p| key(p));
            let o1 = match crash_recovery_disagreement(&committed, persisted_root, &read_back) {
                None => "green".to_string(),
                Some(d) => format!("red:{}", d.model_answer.chars().take(40).collect::<String>().replace(['\t', '\n', ' '], "_")),
            };
            let reffs = match &visible {
                Some(v) => reffs_vote(&reffs_script, v),
                None => "red(recovery_failed)".to_string(),
            };
            let red: Vec<String> = check_pool_image(&pool)
                .into_iter()
                .filter_map(|(name, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then(|| name.to_string()))
                .collect();
            let mount = if do_mount {
                let before = digest(&disks);
                let (first, after, refusal_verdict) = mount_once(&dump, &disks, &pool, report.effective_root);
                let unchanged = before == after;
                let (second, _, _) = mount_once(&dump, &disks, &pool, report.effective_root);
                format!("mount1={first}\tunchanged_after_mount1={unchanged}\tmount2={second}\trefusal_judged_by_crash_injection_second_cut={refusal_verdict}")
            } else {
                "mount=skipped".to_string()
            };
            let visible_text = match &visible {
                Some(VisibleState::NoFile) => "no_file".to_string(),
                Some(VisibleState::OneFile(c)) => format!("file:{}", &sha256_hexadecimal(&c.0)[..12]),
                None => "failed".to_string(),
            };
            writeln!(
                out,
                "{label}\t{state}\tdigest_ok={digest_ok}\trecover={effective}\tread_back={visible_text}\tpersisted={}\to1_crash_judge={o1}\treffs={reffs}\to2_red={}\t{mount}",
                persisted.get(state).map_or("-", String::as_str),
                if red.is_empty() { "-".to_string() } else { red.join(",") }
            )
            .unwrap();
        }
    }
}
