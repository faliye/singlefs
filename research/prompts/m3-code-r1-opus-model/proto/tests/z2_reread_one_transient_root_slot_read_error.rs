//! m3-code-r1 攻方腿 Z2 ④：落后支缺根那一半的「重读一次」（D16 已定项 1「根槽这一次读坏」套到这一槽上）有没有用例钉。
//! 盘面：会话里覆盖写 P（txg 6）做完；P 的根住的那块盘落后（两种造法），单元都在、根槽里就是 P 的根。
//!   甲：那块盘换成「P 之前 + P 的写流取到根槽 FUA 写为止」（根已落、轮换没写；与崩在根槽之后、轮换之前同一个盘面）。
//!   乙：不换盘，P 之后那块盘最新那一槽系统配置坏一个字节（持久单故障），最新自证的那一份停在 P 之前。
//! 再加一次瞬时读错：那块盘上 P 的根槽第 n 次读读不出（n 从 1 扫到干净跑时这一槽被读的次数）。
//! 原样：缺根那一半第一次读坏、重读读出 P，不拒。去掉重读（`let reread = first_reading;`）就拒成落后支——今天没有用例分辨这两种。

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use common_admission::{content_of, plain_devices_on, start_plain, OVERWRITE_BYTES};
use m3r1_opus_proto::{with_faults, FaultRange};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{FirstFile, PoolVersion};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

fn clone_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
    }
}

#[derive(Clone, Copy, Debug)]
enum Behind {
    SwappedForTheCutAfterTheRootSlotWrite,
    NewestSystemConfigurationSlotCorrupted,
}

struct Shape {
    parameters: MakeFilesystemParameters,
    session: MountedSession,
    image: MemoryPool,
    root_device: DeviceIdentity,
    root_slot_offset: u64,
    spacing: u64,
    write_time_seconds: u64,
}

fn shape(behind: Behind) -> Shape {
    let mut pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    pool.overwrite(OVERWRITE_BYTES).expect("txg 4");
    pool.overwrite(OVERWRITE_BYTES).expect("txg 5");
    let before = pool.image();
    let mut session = pool.session.take().expect("会话");
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&before)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let write_time_seconds = pool.write_time_seconds + 1;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    session
        .publish_user_change(&mut recorded, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds }))
        .expect("P");
    let after = MemoryPool {
        devices: recorded.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: before.device_size_in_bytes,
    };
    let PoolVersion::WithFile(current) = &session.current else { panic!("带文件") };
    let geometry = pool.parameters.geometry;
    let ring_slot = target_for_publish(current.root.checkpoint_txg, geometry.root_ring_slots_per_region);
    let root_device = pool.parameters.region_devices[usize::try_from(ring_slot.region).expect("小")];
    let root_slot_offset = slot_offset(ring_slot, geometry.fixed_structure_slot_spacing).0;
    let spacing = u64::from(geometry.fixed_structure_slot_spacing);
    let mut image = after.clone();
    match behind {
        Behind::SwappedForTheCutAfterTheRootSlotWrite => {
            let operations = stream.retained_operations();
            let root_write = operations
                .iter()
                .position(|retained| {
                    retained.operation.device == root_device
                        && retained.operation.offset.0 == root_slot_offset
                        && retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess
                })
                .expect("P 的根槽 FUA 写");
            let mut cut = before.clone();
            cut.apply(&operations[..=root_write]);
            image.devices.insert(root_device, cut.devices[&root_device].clone());
        }
        Behind::NewestSystemConfigurationSlotCorrupted => {
            let devices = plain_devices_on(&after);
            let newest = verified_system_configuration_slots(&devices, root_device, spacing, &pool.parameters.filesystem_identifier)
                .into_iter()
                .max_by_key(|slot| slot.quantities.slot_generation)
                .expect("自证过的");
            // 两槽里世代号大的那一槽：试两槽，挑翻了之后最新自证那一份落后的那一槽。
            let mut chosen = None;
            for slot in 0..2u64 {
                let mut candidate = after.clone();
                candidate.flip_byte(root_device, DeviceOffsetInBytes(slot * spacing), 300);
                let devices = plain_devices_on(&candidate);
                let remaining = verified_system_configuration_slots(&devices, root_device, spacing, &pool.parameters.filesystem_identifier)
                    .into_iter()
                    .max_by_key(|slot| slot.quantities.slot_generation);
                if remaining.is_some_and(|remaining| remaining.quantities.slot_generation < newest.quantities.slot_generation) {
                    chosen = Some(candidate);
                }
            }
            image = chosen.expect("翻最新那一槽之后还剩一份自证、更旧");
        }
    }
    Shape { parameters: pool.parameters.clone(), session, image, root_device, root_slot_offset, spacing, write_time_seconds }
}

fn root_slot_fault(shape: &Shape, failing: &[u64]) -> FaultRange {
    FaultRange { device: shape.root_device, offset: shape.root_slot_offset, length: shape.spacing, failing_read_numbers: failing.to_vec(), reads_so_far: 0 }
}

fn session_publish(shape: &Shape, failing: &[u64]) -> (Result<(), String>, u64) {
    let (mut devices, faults) = with_faults(plain_devices_on(&shape.image), vec![root_slot_fault(shape, failing)]);
    let mut session = clone_session(&shape.session);
    let content = content_of(OVERWRITE_BYTES, shape.write_time_seconds + 1);
    let outcome = session
        .publish_user_change(&mut devices, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds: shape.write_time_seconds + 1 }))
        .map(|_| ())
        .map_err(|error| format!("{error:?}"));
    let reads = faults.borrow().ranges[0].reads_so_far;
    (outcome, reads)
}

fn writable_mount(shape: &Shape, failing: &[u64]) -> (Result<(), String>, u64) {
    let (mut devices, faults) = with_faults(plain_devices_on(&shape.image), vec![root_slot_fault(shape, failing)]);
    let outcome = mount_writable(&shape.parameters, &mut devices).map(|_| ()).map_err(|error| format!("{error:?}"));
    let reads = faults.borrow().ranges[0].reads_so_far;
    (outcome, reads)
}

fn refused_as_behind(outcome: &Result<(), String>) -> bool {
    outcome.as_ref().is_err_and(|text| text.contains("DevicesBehindTheCurrentVersion") || text.contains("BehindTheSelectedVersion"))
}

fn sweep(behind: Behind, entry: &str, run: fn(&Shape, &[u64]) -> (Result<(), String>, u64)) -> Vec<String> {
    let shape = shape(behind);
    let (clean, clean_reads) = run(&shape, &[]);
    println!("{behind:?} {entry} clean={} reads_of_the_root_slot={clean_reads}", clean.as_ref().map_or_else(|e| e.chars().take(140).collect::<String>(), |()| "Ok".into()));
    assert!(clean.is_ok(), "{behind:?} {entry}：没有瞬时读错时照常做成：{clean:?}");
    let mut refused = Vec::new();
    for n in 1..=clean_reads {
        let (outcome, _) = run(&shape, &[n]);
        let text = outcome.as_ref().map_or_else(|e| e.chars().take(140).collect::<String>(), |()| "Ok".into());
        println!("{behind:?} {entry} fail_read={n} outcome={text}");
        if refused_as_behind(&outcome) {
            refused.push(format!("{behind:?} {entry} fail_read={n}: {text}"));
        }
    }
    refused
}

/// 一次瞬时读错不让落后、单元都在、根槽里是它的那块盘被拒成落后支（会话发布与可写挂载，两种落后的造法）。
#[test]
fn one_transient_read_error_on_the_root_slot_does_not_refuse_a_behind_device_holding_the_root() {
    let mut refused = Vec::new();
    for behind in [Behind::SwappedForTheCutAfterTheRootSlotWrite, Behind::NewestSystemConfigurationSlotCorrupted] {
        refused.extend(sweep(behind, "session_publish", session_publish));
        // 乙（最新那一槽系统配置坏）上可写挂载本来就按甲拒（C331：见证读到更新的系统配置），不扫。
        if matches!(behind, Behind::SwappedForTheCutAfterTheRootSlotWrite) {
            refused.extend(sweep(behind, "writable_mount", writable_mount));
        }
    }
    assert!(refused.is_empty(), "一次瞬时读错被拒成落后支：{refused:#?}");
}
