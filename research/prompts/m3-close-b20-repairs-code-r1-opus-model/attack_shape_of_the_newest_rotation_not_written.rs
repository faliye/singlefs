//! 攻方用例（只在副本里）：换造法 `image_with_the_newest_rotation_of_each_device_not_written` 造的盘面。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    image_with_the_newest_rotation_of_each_device_not_written, record_one_publish_after_overwrites,
    PublishRecordedAsR,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::recovery::{choose_system_configuration, verified_system_configuration_slots};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_verification::memory_pool::MemoryPool;
use singlefs_verification::segments::StepKind;

const THE_TWO_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 一块盘两槽里自证过的那几份，按世代号升序。
fn slots_of(image: &MemoryPool, device: DeviceIdentity) -> Vec<SystemConfiguration> {
    let configuration = choose_system_configuration(image).expect("择得到系统配置");
    let mut slots = verified_system_configuration_slots(
        image,
        device,
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing),
        &configuration.immutable.filesystem_identifier,
    );
    slots.sort_by_key(|slot| slot.quantities.slot_generation);
    slots
}

fn highest_tail_of(image: &MemoryPool) -> u64 {
    THE_TWO_DEVICES
        .iter()
        .flat_map(|device| slots_of(image, *device))
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .expect("有自证过的槽")
}

/// 打中面四：用例要的盘面（崩在 R 的根槽之后、轮换之前，又「P 那次轮换没写成」）不是 R 的任何崩溃态。P 的发布返回之前过了末尾屏障（`crates/singlefs-core/src/transaction.rs`
/// 发布的最后三步），P 的轮换写的那一槽在 `base` 上已经持久；R 的写流里系统配置写只落另一槽，碰不到它。于是 R 的每个崩溃态（撕裂在内）c_见证 都不小于 P 的尾；
/// 造法把 c_见证 拉到 P 的尾之下，要两块盘都丢掉一次已确认的写（归 C6（块层语义假设写错））。
/// 另核造法注释「世代号 g - 2、见证值与另一槽相同（那次轮换写下的就是这些）」：同一段历史少覆盖写一次时的较旧那一槽就是真的 g - 2，它的尾与造法写的不同。
#[test]
fn attack_the_fixture_needs_a_lost_write_on_both_devices_and_no_crash_state_of_r_reaches_it() {
    let recorded = record_one_publish_after_overwrites(3, PublishRecordedAsR::Overwrite);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let fixture = image_with_the_newest_rotation_of_each_device_not_written(&crash_image);
    let tail_of_p = highest_tail_of(&recorded.base);
    let tail_on_the_crash_image = highest_tail_of(&crash_image);
    let tail_on_the_fixture = highest_tail_of(&fixture);
    println!(
        "ATTACK_FIXTURE tail_of_p={tail_of_p} tail_on_the_crash_image={tail_on_the_crash_image} tail_on_the_fixture={tail_on_the_fixture}"
    );
    assert_eq!(
        tail_on_the_crash_image, tail_of_p,
        "R 的轮换没落，c_见证 是 P 的尾"
    );
    assert!(
        tail_on_the_fixture < tail_of_p,
        "造法把 c_见证 拉到 P 的尾之下"
    );
    let configuration = choose_system_configuration(&recorded.base).expect("择得到系统配置");
    let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    for device in THE_TWO_DEVICES {
        let newest_on_the_base = slots_of(&recorded.base, device)
            .last()
            .expect("有自证过的槽")
            .quantities
            .slot_generation;
        let offset_of_the_rotation_of_p = (newest_on_the_base % 2) * spacing;
        let rotation_writes_of_r: Vec<u64> = recorded
            .writes
            .iter()
            .filter(|write| {
                write.kind == StepKind::SystemConfigurationSlot && write.device == device
            })
            .map(|write| write.offset.0)
            .collect();
        println!(
            "ATTACK_FIXTURE device={device:?} generation_of_p={newest_on_the_base} offset_of_the_rotation_of_p={offset_of_the_rotation_of_p} rotation_writes_of_r={rotation_writes_of_r:?}"
        );
        assert!(!rotation_writes_of_r.is_empty(), "R 的写流里这块盘有轮换写");
        assert!(
            rotation_writes_of_r
                .iter()
                .all(|offset| *offset != offset_of_the_rotation_of_p),
            "R 的轮换写碰不到 P 的轮换写的那一槽"
        );
    }
    let history_one_overwrite_shorter =
        record_one_publish_after_overwrites(2, PublishRecordedAsR::Overwrite);
    for device in THE_TWO_DEVICES {
        let on_the_fixture = slots_of(&fixture, device);
        let rewritten = on_the_fixture.first().expect("造法写回的那一槽世代号最小");
        let real_rotation = slots_of(&history_one_overwrite_shorter.base, device)
            .first()
            .expect("较旧那一槽")
            .quantities;
        println!(
            "ATTACK_FIXTURE device={device:?} fixture_generation={} fixture_tail={} real_generation={} real_tail={}",
            rewritten.quantities.slot_generation,
            rewritten.quantities.journal_tail,
            real_rotation.slot_generation,
            real_rotation.journal_tail
        );
        assert_eq!(
            rewritten.quantities.slot_generation, real_rotation.slot_generation,
            "同一个世代号"
        );
        assert_ne!(
            rewritten.quantities.journal_tail, real_rotation.journal_tail,
            "造法写的尾不是那次轮换真写下的尾"
        );
    }
}
