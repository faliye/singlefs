//! m2-closeout-code-r3 云端攻方腿 X6：挂着之后的入口「可见」之后核本盘设备号与落后支（落后且缺现行那一版的单元才拒）。
//! 两条：
//! 1. `a_stale_snapshot_taken_mid_publish`：一次发布 P（覆盖写 / 准入抬 F / 管理员回退 / 建一个 inode）做成之后，把盘 d 换成
//!    P 录制流第 k 步之后盘 d 的样子（k 从 0 扫到末尾：0 是 P 之前的整块旧快照，末尾是 P 做完的样子），再做一个用户动作
//!    （会话覆盖写、建一个 inode、准入抬 F、正常卸载、管理员回退、崩了再可写挂载，放开扫），看入口拒不拒；做成的，盘面 checker、
//!    再崩了再可写挂载、checker、读回内容对不对。
//! 2. `one_system_configuration_slot_fault_after_each_publish`：合法的单故障历史——P 做成之后盘 d 的某一个系统配置槽坏一个字节
//!    （两槽各试），再接一串用户动作（放开扫三步），看有没有一步被「可见」或落后支拒掉、有没有 checker 红。

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use std::collections::BTreeMap;

use common_admission::{checker_violations_on, content_of, plain_devices_on, start_plain, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
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
    let cut: String = text.chars().take(160).collect();
    cut.replace('\n', " ")
}

/// 一个动作（P 或用户动作）在给定的盘上跑一次：交回结局与这一次之后现行那一版的内容（没变就是原来的）。
fn act<D: singlefs_core::block_device::BlockDevice>(
    action: &str,
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, D)>,
    session: &mut MountedSession,
    contents: &Contents,
    current_content: &[u8],
    write_time_seconds: u64,
) -> (String, Vec<u8>) {
    match action {
        "overwrite" => {
            let content = content_of(OVERWRITE_BYTES, write_time_seconds);
            let outcome = session.publish_user_change(devices, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds }));
            match outcome {
                Ok(_) => ("Ok".to_string(), content),
                Err(error) => (short(format!("Err {error:?}")), current_content.to_vec()),
            }
        }
        "create_inode" => {
            let outcome = session.publish_user_change(devices, UserChange::NewInodes { count: 1, write_time_seconds });
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, current_content.to_vec())
        }
        "raise_floor" => {
            let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
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
            let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
            let outcome = roll_back_by_a_forward_publish(parameters, devices, &mut session.allocator, current, target);
            match outcome {
                Ok(_) => (format!("Ok to_{}_{}", target.instance.0, target.checkpoint_txg.0), contents[&(target.instance, target.checkpoint_txg)].clone()),
                Err(error) => (short(format!("Err {error:?}")), current_content.to_vec()),
            }
        }
        "crash_remount" => match mount_writable(parameters, devices) {
            Ok(mounted) => {
                let (output, new_session) = MountedSession::of_the_mount(mounted);
                *session = new_session;
                (format!("Ok chose_{}_{}", output.effective_root.instance.0, output.effective_root.checkpoint_txg.0), contents.get(&(output.effective_root.instance, output.effective_root.checkpoint_txg)).cloned().unwrap_or_else(|| current_content.to_vec()))
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

/// 起点：第一个文件之后会话里覆盖写两次（txg 4、5），准入抬 F 一次（F 抬到上限，环里有带 F 的根）。
struct Start {
    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    image: MemoryPool,
    session: MountedSession,
    contents: Contents,
    current_content: Vec<u8>,
    write_time_seconds: u64,
}

fn start() -> Start {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 3");
    let session = pool.session.take().expect("会话");
    Start {
        parameters: pool.parameters.clone(),
        image: pool.image(),
        session,
        contents: pool.content_of_each_root.clone(),
        current_content: pool.content_of_the_current_version.clone(),
        write_time_seconds: pool.write_time_seconds,
    }
}

/// 所有写过的根都记上内容：动作之后根环里新出现、还没记内容的根记成动作之后现行那一版的内容（抬 F、卸载、暖机那几条内容不变；
/// 覆盖写之前为空间推的抬 F 那几条会被记成新内容——只影响「读回内容对不对」那一格的判法，那一格另外拿两份内容都认）。
fn note_new_roots(contents: &mut Contents, image: &MemoryPool, content: &[u8]) {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let roots = singlefs_core::recovery::readable_roots_with_ring_slots(
        image,
        &parameters.region_devices,
        &parameters.geometry,
        &fsid_of(image),
    );
    for (_, root) in roots {
        contents.entry((root.instance, root.checkpoint_txg)).or_insert_with(|| content.to_vec());
    }
}

/// 盘 0 系统配置槽 0 里的 fsid（偏移 4 + 2 + 96，16 字节）。
fn fsid_of(image: &MemoryPool) -> [u8; 16] {
    let bytes = singlefs_core::recovery::PoolReader::read(image, DeviceIdentity(0), DeviceOffsetInBytes(0), 512).expect("盘 0 第一个扇区");
    let mut fsid = [0u8; 16];
    fsid.copy_from_slice(&bytes[4 + 2 + 96..4 + 2 + 96 + 16]);
    fsid
}

const PUBLISHES: [&str; 4] = ["overwrite", "raise_floor", "rollback", "create_inode"];
const ACTIONS: [&str; 6] = ["overwrite", "create_inode", "raise_floor", "unmount", "rollback", "crash_remount"];

#[test]
fn a_stale_snapshot_taken_mid_publish() {
    let start = start();
    let bytes = start.image.device_size_in_bytes;
    let mut accepted_stale = 0u64;
    let mut refused_stale = 0u64;
    let mut flagged = Vec::new();
    for publish in PUBLISHES {
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&start.image)
            .into_iter()
            .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
            .collect();
        let mut session = clone_session(&start.session);
        let (outcome, content_after_p) = act(publish, &start.parameters, &mut devices, &mut session, &start.contents, &start.current_content, start.write_time_seconds + 1);
        assert!(outcome.starts_with("Ok"), "P={publish} 要做成：{outcome}");
        let operations = stream.retained_operations();
        let image_after_p = MemoryPool {
            devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
            device_size_in_bytes: bytes,
        };
        let mut contents = start.contents.clone();
        note_new_roots(&mut contents, &image_after_p, &content_after_p);
        println!("P={publish} outcome={outcome} operations={} current=({},{})", operations.len(), session.current.root().instance.0, session.current.root().checkpoint_txg.0);
        for stale_device in [DeviceIdentity(0), DeviceIdentity(1)] {
            for k in 0..operations.len() {
                let mut partial = start.image.clone();
                partial.apply(&operations[..k]);
                let stale_image = partial.devices[&stale_device].clone();
                if stale_image.written_sectors() == image_after_p.devices[&stale_device].written_sectors() {
                    continue;
                }
                let kind_k = if k == 0 { "-".to_string() } else { format!("{:?}", operations[k - 1].operation.kind) };
                for action in ACTIONS {
                    let mut image = image_after_p.clone();
                    image.devices.insert(stale_device, stale_image.clone());
                    let mut devices = plain_devices_on(&image);
                    let mut action_session = clone_session(&session);
                    let (action_outcome, content_after) = act(action, &start.parameters, &mut devices, &mut action_session, &contents, &content_after_p, start.write_time_seconds + 2);
                    let refused_as_stale = action_outcome.contains("DevicesBehindTheCurrentVersionAndMissingItsUnits") || action_outcome.contains("CallerInputsDisagree") || action_outcome.contains("DevicesWithout") || action_outcome.contains("CallerParametersDisagree") || action_outcome.contains("WritableMountRefusedByDevicesWithoutTheSelectedVersion");
                    let after = image_of_plain(&devices, bytes);
                    let changed = after.devices.iter().any(|(identity, device)| device.written_sectors() != image.devices[identity].written_sectors());
                    let mut line = format!("STALE P={publish} stale_device={} k={k} last_op={kind_k} action={action} outcome={action_outcome} wrote={changed}", stale_device.0);
                    if refused_as_stale {
                        refused_stale += 1;
                        if changed {
                            flagged.push(format!("{line} refused_after_writing"));
                        }
                    } else if action_outcome.starts_with("Ok") {
                        accepted_stale += 1;
                        let violations = checker_violations_on(&after);
                        let mut remount_devices = plain_devices_on(&after);
                        let remount = mount_writable(&start.parameters, &mut remount_devices);
                        let (remount_text, violations_after, read) = match remount {
                            Ok(_) => {
                                let remounted = image_of_plain(&remount_devices, bytes);
                                ("Ok".to_string(), checker_violations_on(&remounted), read_back(&remounted))
                            }
                            Err(error) => (short(format!("Err {error:?}")), Vec::new(), read_back(&after)),
                        };
                        let content_ok = match (&read.0, &read.1) {
                            (Some(root), Some(content)) => *content == content_after || contents.get(root).is_some_and(|known| known == content),
                            _ => false,
                        };
                        line = format!("{line} checker={} first={:?} remount={remount_text} checker_after={} first_after={:?} read={} root={:?} content_ok={content_ok}", violations.len(), violations.first(), violations_after.len(), violations_after.first(), read.2, read.0);
                        if !violations.is_empty() || !violations_after.is_empty() || remount_text != "Ok" || !content_ok {
                            flagged.push(line.clone());
                        }
                    }
                    println!("{line}");
                }
            }
        }
    }
    println!("STALE accepted={accepted_stale} refused={refused_stale} flagged={}", flagged.len());
    for line in &flagged {
        println!("FLAG {line}");
    }
}

#[test]
fn one_system_configuration_slot_fault_after_each_publish() {
    let start = start();
    let bytes = start.image.device_size_in_bytes;
    let spacing = u64::from(start.parameters.geometry.fixed_structure_slot_spacing);
    let mut flagged = Vec::new();
    let mut sequences = 0u64;
    for publish in PUBLISHES {
        let mut devices = plain_devices_on(&start.image);
        let mut session = clone_session(&start.session);
        let (outcome, content_after_p) = act(publish, &start.parameters, &mut devices, &mut session, &start.contents, &start.current_content, start.write_time_seconds + 1);
        assert!(outcome.starts_with("Ok"), "P={publish} 要做成：{outcome}");
        let image_after_p = image_of_plain(&devices, bytes);
        let mut contents = start.contents.clone();
        note_new_roots(&mut contents, &image_after_p, &content_after_p);
        for faulty_device in [DeviceIdentity(0), DeviceIdentity(1)] {
            for slot in 0..2u64 {
                for first in ACTIONS {
                    for second in ACTIONS {
                        for third in ["overwrite", "create_inode", "unmount"] {
                            sequences += 1;
                            // 每一串从 P 之后记的那一份内容表起：不同串里同一个 (实例, txg) 是不同的发布，内容表不跨串共用。
                            let mut contents = contents.clone();
                            let mut image = image_after_p.clone();
                            image.flip_byte(faulty_device, DeviceOffsetInBytes(slot * spacing), 300);
                            let mut devices = plain_devices_on(&image);
                            let mut session_now = clone_session(&session);
                            let mut content_now = content_after_p.clone();
                            let mut outcomes = Vec::new();
                            let mut stop = false;
                            for (index, action) in [first, second, third].into_iter().enumerate() {
                                if stop {
                                    break;
                                }
                                let (action_outcome, content_after) = act(action, &start.parameters, &mut devices, &mut session_now, &contents, &content_now, start.write_time_seconds + 2 + u64::try_from(index).expect("小"));
                                if action_outcome.starts_with("Ok") {
                                    content_now = content_after;
                                    let image_now = image_of_plain(&devices, bytes);
                                    note_new_roots(&mut contents, &image_now, &content_now);
                                }
                                let refused_as_stale = action_outcome.contains("DevicesBehindTheCurrentVersionAndMissingItsUnits") || action_outcome.contains("DevicesWithout") || action_outcome.contains("CallerParametersDisagree") || action_outcome.contains("CallerInputsDisagree") || action_outcome.contains("OwnDeviceNumbers");
                                if refused_as_stale {
                                    flagged.push(format!("ONE_SLOT P={publish} device={} slot={slot} step={index} action={action} outcome={action_outcome}", faulty_device.0));
                                }
                                if action == "unmount" && action_outcome.starts_with("Ok") {
                                    stop = true;
                                }
                                outcomes.push(format!("{action}:{}", action_outcome.chars().take(60).collect::<String>()));
                            }
                            let after = image_of_plain(&devices, bytes);
                            let violations = checker_violations_on(&after);
                            let mut remount_devices = plain_devices_on(&after);
                            let remount = match mount_writable(&start.parameters, &mut remount_devices) { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) };
                            let read = read_back(&after);
                            let content_ok = read.1.as_ref().is_some_and(|content| *content == content_now || read.0.and_then(|root| contents.get(&root)).is_some_and(|known| known == content));
                            let line = format!("ONE_SLOT P={publish} device={} slot={slot} seq={outcomes:?} checker={} first={:?} remount={remount} read={} content_ok={content_ok}", faulty_device.0, violations.len(), violations.first(), read.2);
                            if !violations.is_empty() || !content_ok {
                                flagged.push(line.clone());
                            }
                            println!("{line}");
                        }
                    }
                }
            }
        }
    }
    println!("ONE_SLOT sequences={sequences} flagged={}", flagged.len());
    for line in &flagged {
        println!("FLAG {line}");
    }
}

/// 查 `one_system_configuration_slot_fault_after_each_publish` 里 content_ok=false 那几格是不是装置记错了内容：
/// 同一串动作，盘 0 槽 0 坏与不坏各跑一遍，打出读回的根、内容长度、与 P 那一版内容 / 起点那一版内容是否相等。
#[test]
fn content_check_of_the_one_slot_sequences_with_a_crash_remount() {
    let start = start();
    let bytes = start.image.device_size_in_bytes;
    for corrupt in [false, true] {
        let mut devices = plain_devices_on(&start.image);
        let mut session = clone_session(&start.session);
        let (_, content_after_p) = act("overwrite", &start.parameters, &mut devices, &mut session, &start.contents, &start.current_content, start.write_time_seconds + 1);
        if corrupt {
            let mut image = image_of_plain(&devices, bytes);
            image.flip_byte(DeviceIdentity(0), DeviceOffsetInBytes(0), 300);
            devices = plain_devices_on(&image);
        }
        let mut contents = start.contents.clone();
        note_new_roots(&mut contents, &image_of_plain(&devices, bytes), &content_after_p);
        let mut content_now = content_after_p.clone();
        for (index, action) in ["create_inode", "crash_remount", "create_inode"].into_iter().enumerate() {
            let (outcome, content_after) = act(action, &start.parameters, &mut devices, &mut session, &contents, &content_now, start.write_time_seconds + 2 + u64::try_from(index).expect("小"));
            content_now = content_after;
            note_new_roots(&mut contents, &image_of_plain(&devices, bytes), &content_now);
            let (root, content, text) = read_back(&image_of_plain(&devices, bytes));
            println!(
                "DEBUG corrupt={corrupt} step={index} action={action} outcome={} read={text} root={root:?} len={:?} equals_p={} equals_start={} equals_content_now={}",
                outcome.chars().take(60).collect::<String>(),
                content.as_ref().map(Vec::len),
                content.as_ref().is_some_and(|content| *content == content_after_p),
                content.as_ref().is_some_and(|content| *content == start.current_content),
                content.as_ref().is_some_and(|content| *content == content_now)
            );
        }
    }
}
