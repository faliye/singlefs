//! 正推腿探针（第 23 批代码轮第一轮 K3）：`writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 第 287–290 行那句注释的事实：
//! 盘 0 两槽都撕裂的盘面上，c_见证 取自盘 1 自证过的槽、等于第一个文件那一版末条记录的计数器。
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::recovery::{
    choose_system_configuration, recover, scan_journal, verified_system_configuration_slots,
    JournalPolicy, PoolReader,
};
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
    HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::{SharedStream};

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;

#[test]
fn witnessed_counter_on_the_image_with_both_slots_of_device_zero_torn() {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: Vec::new(),
    };
    let _ = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: SMALL,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |_| {},
    );
    let mut healthy = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        SMALL.device_bytes(),
    );
    healthy.apply(&stream.retained_operations());
    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).unwrap();
    let mut torn = healthy.clone();
    for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot * slot_spacing);
        let mut bytes = healthy.read(DeviceIdentity(0), offset, slot_bytes).unwrap();
        bytes[2048] ^= 0xFF;
        torn.devices.get_mut(&DeviceIdentity(0)).unwrap().write(offset, &bytes);
    }
    for (name, image) in [("健康盘面", &healthy), ("盘0两槽撕裂", &torn)] {
        let system_configuration = choose_system_configuration(image).expect("择得到系统配置");
        let fsid = system_configuration.immutable.filesystem_identifier;
        let mut per_device = Vec::new();
        for device in image.device_identities() {
            let tails: Vec<u64> = verified_system_configuration_slots(image, device, slot_spacing, &fsid)
                .into_iter().map(|slot| slot.quantities.journal_tail)
                .collect();
            per_device.push((device, tails));
        }
        let witnessed = per_device.iter().flat_map(|(_, t)| t.iter().copied()).max().unwrap_or(0);
        let selected = recover(image, JournalPolicy::Consult).effective_root;
        let records = scan_journal(image, &system_configuration);
        let last_of_selected = records
            .values()
            .filter(|r| Some((r.instance, r.checkpoint_txg)) == selected
                && r.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish)
            .map(|r| r.counter)
            .max();
        println!(
            "[{name}] 自证过的槽 journal_tail 按盘 {per_device:?}；c_见证={witnessed}；所选那一版 {selected:?} 末条记录计数器 {last_of_selected:?}；c_见证 > 末条={}",
            last_of_selected.map(|c| witnessed > c).unwrap_or(true)
        );
    }
}
