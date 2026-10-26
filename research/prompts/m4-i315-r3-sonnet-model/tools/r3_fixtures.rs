//! 辩方腿（m4-i315-r3）改自第一轮正推腿的入库固定盘面探针：每张盘按九个臂各判一遍。原探针（m4-i315-r1）：把 harness 场景交给 checker 档的隔离相关镜像收下来，在 CPU 池级 checker 上逐张判，打 I-3.1 / I-3.11 / I-3.15 的判定。
//! 不改任何断言；开关由 walk.rs 里的环境变量定（PROBE_FIX_GROUPING / PROBE_FIX_GROUPING_WHEN_NO_KEY）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;

use common_images_for_the_checker_tier::PoolCheckerImages;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, defense_set_arm};

fn print_the_images(scenario: &str, images: PoolCheckerImages) {
    let PoolCheckerImages::CollectedForTheCheckerTier(collected) = images else {
        panic!("要收下镜像");
    };
    for image in collected {
      for arm in ["today", "exempt", "gen", "genplus", "cmp", "both", "bothplus", "jrn", "jrngen"] {
        defense_set_arm(arm);
        let verdicts = check_pool_image(&image.image);
        let show = |name: &str| {
            verdicts
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| match v {
                    InvariantVerdict::Holds => "成立".to_string(),
                    InvariantVerdict::Violated(d) => format!("违例({})", d.chars().take(60).collect::<String>()),
                    InvariantVerdict::NotApplicable(r) => format!("不适用({r})"),
                    InvariantVerdict::PartiallyJudged(_) => "部分没判".to_string(),
                })
                .unwrap_or_else(|| "无此条".to_string())
        };
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_)))
            .map(|(n, _)| *n)
            .collect();
        println!(
            "SCEN arm={arm} {scenario} / {} : I-3.1={} ; I-3.11={} ; I-3.15={} ; 全部违例={violated:?}",
            image.label,
            show("I-3.1"),
            show("I-3.11"),
            show("I-3.15"),
        );
      }
    }
    defense_set_arm("today");
}

#[test]
fn quarantine_scenario_images_judged_by_the_cpu_pool_checker() {
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(&mut images);
    print_the_images("两盘改坏/改回", images);
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not(&mut images);
    print_the_images("只盘0改坏/改回", images);
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(&mut images);
    print_the_images("完整的别的单元", images);
}


use common::{build_pool, parameters, publish_overwrite_in_process, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use scenarios_of_release_checksum_quarantine::{content_of, data_slot_of_the_first_version};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{raise_rollback_floor, ShadowLedger};

/// 头所在的前 4096 字节写零（头用不了），或整个 32 KiB 单元写 0xa5。
fn spoil_the_header_on(devices: &mut [(DeviceIdentity, Recorded)], device: DeviceIdentity, slot: SlotNumber, whole_unit: bool) {
    let (_, block_device) = devices.iter_mut().find(|(identity, _)| *identity == device).expect("池里有这块盘");
    let bytes = if whole_unit { vec![0xa5u8; 32768] } else { vec![0u8; 4096] };
    block_device
        .write_at(DeviceOffsetInBytes(slot.to_device_offset().0), &bytes, WriteDurability::Plain)
        .expect("写坏头");
}

/// 造一个世界：A 的数据单元在 `spoiled_devices` 那几块盘上头用不了，覆盖写 B 隔离它；`further_publishes` 再覆盖写几次；`raise_floor_to` 给了就抬 F。
fn world(tag: &str, spoiled_devices: &[DeviceIdentity], whole_unit: bool, further_publishes: usize, raise_floor_to: Option<u64>) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for device in spoiled_devices {
            spoil_the_header_on(devices, *device, data_slot, whole_unit);
        }
    }
    let mut current = publish_overwrite_in_process(&mut pool, &first, &content_of(2800, 5), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1)).expect("核出对不上时发布照成");
    println!("WORLD {tag}: 隔离 {} 份", current.quarantined_after_release_checksum_mismatch.len());
    for k in 0..further_publishes {
        current = publish_overwrite_in_process(&mut pool, &current, &content_of(2700, 6 + k), FIXED_WRITE_TIME_SECONDS + 120 + 60 * k as u64, InstanceGeneration(1)).expect("接着覆盖写");
    }
    if let Some(floor) = raise_floor_to {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        raise_rollback_floor(&publish_parameters, devices, &mut pool.allocator, &mut current, CheckpointTxg(floor), ShadowLedger::On).expect("抬 F");
    }
    pool
}

#[test]
fn header_unusable_quarantine_worlds_judged_by_the_cpu_pool_checker() {
    let d0 = DeviceIdentity(0);
    let d1 = DeviceIdentity(1);
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    for (label, pool) in [
        ("W1 盘0头写零、盘1好、B 一次、A 仍是候选", world("p1", &[d0], false, 0, None)),
        ("W1b 盘1头写零、盘0好、B 一次、A 仍是候选", world("p1b", &[d1], false, 0, None)),
        ("W2 两盘头写零、B 一次、A 仍是候选", world("p2", &[d0, d1], false, 0, None)),
        ("W3 盘0整单元0xa5、盘1好、再三次覆盖写并抬 F 到 4（A 出候选集）", world("p3", &[d0], true, 3, Some(4))),
        ("W3b 同 W3 但只再覆盖写一次、不抬 F", world("p3b", &[d0], true, 1, None)),
        ("W0 无损坏（对照：隔离不发生）", world("p0", &[], false, 0, None)),
    ] {
        let image = pool.memory_pool();
        images.hand_out(label, || image.clone(), common_images_for_the_checker_tier::PoolCheckerExpectation::only_the_named_invariants());
    }
    print_the_images("头用不了的世界", images);
}

use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};

/// 合法历史：A 的数据单元盘 1 那一份头用不了，覆盖写 B 隔离它；之后（甲）原样不动、（乙）盘 1 那一份写回原样、（丙）写回原样再回退到 A。
/// （乙）那张盘上没有 key、两份都好；（丙）那张盘上又有 key 指着那一槽（隔离之后某一版仍有 key 指着）。
#[test]
fn isolated_unit_repaired_and_then_revived_by_a_rollback_judged_by_the_cpu_pool_checker() {
    let d0 = DeviceIdentity(0);
    let d1 = DeviceIdentity(1);
    let mut pool = build_pool("revive");
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    let intact = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let (_, block_device) = devices.iter().find(|(identity, _)| *identity == d1).expect("盘 1");
        let mut bytes = vec![0u8; 32768];
        block_device.read_at(data_slot.to_device_offset(), &mut bytes).expect("读");
        bytes
    };
    spoil_the_header_on(pool.devices.as_mut().expect("镜像还开着"), d1, data_slot, false);
    let second = publish_overwrite_in_process(&mut pool, &first, &content_of(2700, 9), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1)).expect("覆盖写照成");
    println!("WORLD revive: 隔离 {} 份", second.quarantined_after_release_checksum_mismatch.len());
    pool.output = second;
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    let image_a = pool.memory_pool();
    images.hand_out("甲 隔离着、盘 1 头仍用不了", || image_a.clone(), common_images_for_the_checker_tier::PoolCheckerExpectation::only_the_named_invariants());
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let (_, block_device) = devices.iter_mut().find(|(identity, _)| *identity == d1).expect("盘 1");
        block_device.write_at(data_slot.to_device_offset(), &intact, WriteDurability::Plain).expect("写回原样");
    }
    let image_b = pool.memory_pool();
    images.hand_out("乙 盘 1 写回原样、未回退（无 key、两份都好）", || image_b.clone(), common_images_for_the_checker_tier::PoolCheckerExpectation::only_the_named_invariants());
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let rolled = roll_back_by_a_forward_publish(&publish_parameters, devices, &mut pool.allocator, &mut pool.output, RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) });
    println!("WORLD revive: 回退 {}", if rolled.is_ok() { "做成" } else { "被拒" });
    let image_c = pool.memory_pool();
    images.hand_out("丙 写回原样后回退到 A（又有 key 指着那一槽）", || image_c.clone(), common_images_for_the_checker_tier::PoolCheckerExpectation::only_the_named_invariants());
    print_the_images("隔离之后又被回退复活", images);
}
