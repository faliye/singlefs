//! m3-code-r1 攻方腿 Z2 ①：旧快照取自一段多次发布历史里的任一次发布途中（不只最后那一次），换上来之后放开扫一到两个用户动作。
//! 实现员列的零轮候选 3（旧快照取在更早一次发布途中、之后又发过几次，现行那一版的根不住在这块盘上）与 4（P 的根住在另一块盘上、
//! 被换的盘是 P 途中取下的、单元都在）都落在这张表里：历史 = 起点之后 m 次发布（每次从 覆盖写 / 准入抬 F / 管理员回退 / 建一个 inode 里取，
//! m = 1..=M3R1_MAX_PUBLISHES 全排列），旧快照 = 起点 + 整段历史录制流的前 k 步之后盘 d 的样子（k 扫遍整段流、只留 d 的盘面变了的那几个 k），
//! 换上来之后接一个动作（6 种），做成的再接第二个动作（6 种）；只固定前缀（历史）与故障（换盘），用户动作放开扫。
//! 判：入口拒不拒、拒在写之前没有；放过的，每一步之后池级 checker、最后崩了再可写挂载、checker、读回内容对不对。
//! 分片：M3R1_SHARD=i/N 只跑历史编号模 N 等于 i 的那几段（几个进程并行时各跑一片）。
//! 起点照 m2-closeout-code-r3 攻方 X6（384 槽单元区、第一个文件之后会话里覆盖写三次）。

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use std::collections::BTreeMap;

use common_admission::{checker_violations_on, content_of, plain_devices_on, start_plain, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, unmount,
    RaiseToTheAdmissionCeiling, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{FirstFile, PoolVersion};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Contents = BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>;

fn clone_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
    }
}

fn short(text: String) -> String {
    let cut: String = text.chars().take(200).collect();
    cut.replace('\n', " ")
}

fn act<D: BlockDevice>(
    action: &str,
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, D)>,
    session: &mut MountedSession,
    contents: &Contents,
    current_content: &[u8],
    write_time_seconds: u64,
) -> (String, Vec<u8>) {
    match action {
        "overwrite" => {
            let content = content_of(OVERWRITE_BYTES, write_time_seconds);
            match session.publish_user_change(devices, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds })) {
                Ok(_) => ("Ok".to_string(), content),
                Err(error) => (short(format!("Err {error:?}")), current_content.to_vec()),
            }
        }
        "create_inode" => {
            let outcome = session.publish_user_change(devices, UserChange::NewInodes { count: 1, write_time_seconds });
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, current_content.to_vec())
        }
        "raise_floor" => {
            let PoolVersion::WithFile(current) = &mut session.current else { return ("NoFile".to_string(), current_content.to_vec()) };
            let outcome = raise_rollback_floor_to_the_admission_ceiling(parameters, devices, &mut session.allocator, current, ShadowLedger::On);
            (match outcome {
                Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => format!("Ok Raised_to_{}", raised.ceiling.0),
                Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. }) => "Ok AtCeiling".to_string(),
                Err(error) => short(format!("Err {error:?}")),
            }, current_content.to_vec())
        }
        "unmount" => {
            let outcome = unmount(parameters, devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, current_content.to_vec())
        }
        "rollback" => {
            let current_root = *session.current.root();
            let target = contents
                .keys()
                .filter(|(_, txg)| *txg >= current_root.rollback_floor && *txg < current_root.checkpoint_txg)
                .min_by_key(|(_, txg)| *txg)
                .map(|(instance, checkpoint_txg)| RollbackTarget { instance: *instance, checkpoint_txg: *checkpoint_txg });
            let Some(target) = target else { return ("NoCandidate".to_string(), current_content.to_vec()) };
            let PoolVersion::WithFile(current) = &mut session.current else { return ("NoFile".to_string(), current_content.to_vec()) };
            match roll_back_by_a_forward_publish(parameters, devices, &mut session.allocator, current, target) {
                Ok(_) => (format!("Ok to_{}_{}", target.instance.0, target.checkpoint_txg.0), contents[&(target.instance, target.checkpoint_txg)].clone()),
                Err(error) => (short(format!("Err {error:?}")), current_content.to_vec()),
            }
        }
        "crash_remount" => match mount_writable(parameters, devices) {
            Ok(mounted) => {
                let (output, new_session) = MountedSession::of_the_mount(mounted);
                *session = new_session;
                (
                    format!("Ok chose_{}_{}", output.effective_root.instance.0, output.effective_root.checkpoint_txg.0),
                    contents.get(&(output.effective_root.instance, output.effective_root.checkpoint_txg)).cloned().unwrap_or_else(|| current_content.to_vec()),
                )
            }
            Err(error) => (short(format!("Err {error:?}")), current_content.to_vec()),
        },
        other => panic!("没有这个动作 {other}"),
    }
}

fn image_of_plain(devices: &[(DeviceIdentity, SparseBlockDevice)], bytes: u64) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(), device_size_in_bytes: bytes }
}

fn read_back(image: &MemoryPool) -> (Option<(InstanceGeneration, CheckpointTxg)>, Option<Vec<u8>>, String) {
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => (Some(root), Some(content), "FileRead".to_string()),
        RecoveryOutcome::NoFile { root } => (Some(root), None, "NoFile".to_string()),
        RecoveryOutcome::Failed { root, failure } => (root, None, short(format!("Failed {failure:?}"))),
    }
}

fn fsid_of(image: &MemoryPool) -> [u8; 16] {
    let bytes = singlefs_core::recovery::PoolReader::read(image, DeviceIdentity(0), DeviceOffsetInBytes(0), 512).expect("盘 0 第一个扇区");
    let mut fsid = [0u8; 16];
    fsid.copy_from_slice(&bytes[4 + 2 + 96..4 + 2 + 96 + 16]);
    fsid
}

fn note_new_roots(contents: &mut Contents, image: &MemoryPool, content: &[u8]) {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let roots = singlefs_core::recovery::readable_roots_with_ring_slots(image, &parameters.region_devices, &parameters.geometry, &fsid_of(image));
    // 只记带文件的根：mkfs 与暖机那几条（树表 0 条）不进内容表，回退选目标时不会选到它们（第五批之前的版本把它们也记上了，
    // 第二个位置起的回退都选到 (0, 0) 那条、被拒成 VersionWithoutFile，那几段历史整段跳过）。
    for (_, root) in roots {
        if root.checkpoint_txg.0 >= 3 {
            contents.entry((root.instance, root.checkpoint_txg)).or_insert_with(|| content.to_vec());
        }
    }
}

fn refused_as_stale(outcome: &str) -> bool {
    outcome.contains("DevicesBehindTheCurrentVersion")
        || outcome.contains("CallerInputsDisagree")
        || outcome.contains("DevicesWithout")
        || outcome.contains("CallerParametersDisagree")
        || outcome.contains("WritableMountRefusedByDevicesWithoutTheSelectedVersion")
        || outcome.contains("OwnDeviceNumbers")
}

const PUBLISHES: [&str; 4] = ["overwrite", "raise_floor", "rollback", "create_inode"];
const ACTIONS: [&str; 6] = ["overwrite", "create_inode", "raise_floor", "unmount", "rollback", "crash_remount"];

fn histories(max: usize) -> Vec<Vec<&'static str>> {
    let mut all: Vec<Vec<&'static str>> = vec![vec![]];
    let mut out = Vec::new();
    for _ in 0..max {
        let mut next = Vec::new();
        for prefix in &all {
            for publish in PUBLISHES {
                let mut history = prefix.clone();
                history.push(publish);
                out.push(history.clone());
                next.push(history);
            }
        }
        all = next;
    }
    out
}

#[test]
fn a_stale_snapshot_from_any_publish_of_a_history_then_one_or_two_user_actions() {
    let max_publishes: usize = std::env::var("M3R1_MAX_PUBLISHES").ok().and_then(|text| text.parse().ok()).unwrap_or(2);
    let (shard, shards): (usize, usize) = std::env::var("M3R1_SHARD")
        .ok()
        .and_then(|text| text.split_once('/').map(|(a, b)| (a.parse().expect("片号"), b.parse().expect("片数"))))
        .unwrap_or((0, 1));
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 3");
    // M3R1_PREFIX_OVERWRITES：起点再多覆盖写几次，让根环转过一圈（384 槽几何每区 8 槽、三区 24 槽），旧快照上 P 的根槽里是上一圈的根。
    let prefix: usize = std::env::var("M3R1_PREFIX_OVERWRITES").ok().and_then(|text| text.parse().ok()).unwrap_or(0);
    for _ in 0..prefix {
        pool.overwrite(OVERWRITE_BYTES).expect("前缀覆盖写");
    }
    let parameters = pool.parameters.clone();
    let start_image = pool.image();
    let bytes = start_image.device_size_in_bytes;
    let start_session = pool.session.take().expect("会话");
    let start_contents = pool.content_of_each_root.clone();
    let start_content = pool.content_of_the_current_version.clone();
    let mut time = pool.write_time_seconds;
    let (mut cells, mut accepted, mut refused, mut accepted_from_an_earlier_publish) = (0u64, 0u64, 0u64, 0u64);
    let mut flagged = Vec::new();
    for (history_number, history) in histories(max_publishes).into_iter().enumerate() {
        if history_number % shards != shard {
            continue;
        }
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&start_image)
            .into_iter()
            .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
            .collect();
        let mut session = clone_session(&start_session);
        let mut contents = start_contents.clone();
        let mut content_now = start_content.clone();
        let mut publish_ends = Vec::new();
        let mut history_ok = true;
        for publish in &history {
            time += 1;
            let (outcome, content_after) = act(publish, &parameters, &mut devices, &mut session, &contents, &content_now, time);
            if !outcome.starts_with("Ok") {
                println!("HISTORY {history:?} publish={publish} outcome={outcome} 历史做不成，跳过");
                history_ok = false;
                break;
            }
            content_now = content_after;
            let image_now = MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(), device_size_in_bytes: bytes };
            note_new_roots(&mut contents, &image_now, &content_now);
            publish_ends.push(stream.retained_operations().len());
        }
        if !history_ok {
            continue;
        }
        let operations = stream.retained_operations();
        let image_after = MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(), device_size_in_bytes: bytes };
        println!("HISTORY {history:?} operations={} publish_ends={publish_ends:?} current=({},{})", operations.len(), session.current.root().instance.0, session.current.root().checkpoint_txg.0);
        for stale_device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let mut previous_stale = None;
            for k in 0..operations.len() {
                let mut partial = start_image.clone();
                partial.apply(&operations[..k]);
                let stale_image = partial.devices[&stale_device].clone();
                if previous_stale.as_ref().is_some_and(|previous: &singlefs_harness::memory_pool::SparseDevice| previous.written_sectors() == stale_image.written_sectors()) {
                    continue;
                }
                previous_stale = Some(stale_image.clone());
                if stale_image.written_sectors() == image_after.devices[&stale_device].written_sectors() {
                    continue;
                }
                let in_publish = publish_ends.iter().position(|end| k < *end).expect("k 在流里");
                let is_last = in_publish + 1 == history.len();
                for first in ACTIONS {
                    for second in std::iter::once("-").chain(ACTIONS) {
                        if second != "-" && first == "unmount" {
                            continue;
                        }
                        cells += 1;
                        let mut image = image_after.clone();
                        image.devices.insert(stale_device, stale_image.clone());
                        let mut devices = plain_devices_on(&image);
                        let mut action_session = clone_session(&session);
                        let mut contents_now = contents.clone();
                        let (first_outcome, mut content_after) = act(first, &parameters, &mut devices, &mut action_session, &contents_now, &content_now, time + 1);
                        let after_first = image_of_plain(&devices, bytes);
                        let wrote = after_first.devices.iter().any(|(identity, device)| device.written_sectors() != image.devices[identity].written_sectors());
                        let mut line = format!("STALE h={history:?} d={} k={k} in_publish={in_publish} last={is_last} first={first} second={second} first_outcome={first_outcome} wrote={wrote}", stale_device.0);
                        if refused_as_stale(&first_outcome) {
                            if second == "-" {
                                refused += 1;
                                if wrote {
                                    flagged.push(format!("{line} refused_after_writing"));
                                }
                            }
                            continue;
                        }
                        if !first_outcome.starts_with("Ok") {
                            if second == "-" { println!("{line}"); }
                            continue;
                        }
                        if second == "-" {
                            accepted += 1;
                            if !is_last {
                                accepted_from_an_earlier_publish += 1;
                            }
                        }
                        note_new_roots(&mut contents_now, &after_first, &content_after);
                        let violations_first = checker_violations_on(&after_first);
                        let mut second_outcome = "-".to_string();
                        let mut after = after_first.clone();
                        if second != "-" {
                            let (outcome, content) = act(second, &parameters, &mut devices, &mut action_session, &contents_now, &content_after, time + 2);
                            second_outcome = outcome;
                            if second_outcome.starts_with("Ok") {
                                content_after = content;
                            }
                            after = image_of_plain(&devices, bytes);
                            note_new_roots(&mut contents_now, &after, &content_after);
                        }
                        let violations = checker_violations_on(&after);
                        let mut remount_devices = plain_devices_on(&after);
                        let (remount_text, violations_after, read) = match mount_writable(&parameters, &mut remount_devices) {
                            Ok(_) => {
                                let remounted = image_of_plain(&remount_devices, bytes);
                                ("Ok".to_string(), checker_violations_on(&remounted), read_back(&remounted))
                            }
                            Err(error) => (short(format!("Err {error:?}")), Vec::new(), read_back(&after)),
                        };
                        let content_ok = match (&read.0, &read.1) {
                            (Some(root), Some(content)) => *content == content_after || contents_now.get(root).is_some_and(|known| known == content),
                            _ => false,
                        };
                        line = format!("{line} checker_first={} second_outcome={second_outcome} checker={} first_violation={:?} remount={remount_text} checker_after={} first_after={:?} read={} root={:?} content_ok={content_ok}", violations_first.len(), violations.len(), violations.first().or(violations_first.first()), violations_after.len(), violations_after.first(), read.2, read.0);
                        if !violations_first.is_empty() || !violations.is_empty() || !violations_after.is_empty() || remount_text != "Ok" || !content_ok {
                            flagged.push(line.clone());
                        }
                        if second == "-" || !line.contains("content_ok=true") { println!("{line}"); }
                    }
                }
            }
        }
    }
    println!("SUMMARY prefix_overwrites={prefix} shard={shard}/{shards} max_publishes={max_publishes} cells={cells} accepted={accepted} accepted_from_an_earlier_publish={accepted_from_an_earlier_publish} refused={refused} flagged={}", flagged.len());
    for line in &flagged {
        println!("FLAG {line}");
    }
}
