//! 原来同取一圈窗口的六处里另两处，用户 2026-10-04 定 RGall 一起退回整个环（D16（发布语义） 已定项 1；
//! `research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）：影子账的候选过滤（`mount::isolate_slots_referenced_only_by_abandoned_roots`：
//! 按实例表有效 ∧ txg ≥ F 的根都是候选，窗口外的有效根照样豁免它引用的槽）与池级 checker I-3.1（已分配统计对得上） 丁-defer 的门槛
//! （`walk::deferred_slots_reclaim_threshold`：max(F_生效, 环里最旧有效根)）。改回一圈窗口时这两处没有会红的用例
//! （判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二节 K1「影子账候选、I-3.1 丁-defer 门槛改回窗口」那一行，正推 K1-2）。
//!
//! 盘面是 S = 4 的 12 槽小环（`common_root_ring_holes`）。两形都先造一条被抛弃的根（实例 1 最新那次发布的系统配置轮换没落、下一次可写挂载时它的
//! 根槽与数据单元一时读不出，择根落到前一条根 B、实例 2 写行，之后它读得出、按实例表被抛弃），再让一次可写挂载崩在写行那次根槽写之前
//! （K1 那一类洞：txg 跳一个，洞那一槽里留着上一圈的根），之后的发布把锚往前推，洞里那条根落到一圈窗口外、仍是有效根。
//! - 丁-defer（`raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs` 那一形搬到有洞的小环上）：B 那一槽在洞里留着，
//!   F 抬到 B 与被抛弃的那条根之间（B 在 F 之下）。B 引用的实例 1 那张实例表在实例 2 写行那次被整条重写，释放代高于 F（还在 defer 里，分配器照算已分配），
//!   F 之上的有效根都不引用它，被抛弃的那条根不走、它的记录也不算被施加过。门槛取整个环是 max(F, 环里最旧有效根 B) = F，并进遍历；
//!   照一圈窗口取时门槛是窗口里最旧的有效根（高于那个释放代），那几槽不并，I-3.1 红在记账多的一边。
//!   不用 K1 那一形的「实例 1 连崩之后 F 抬到 24」：那里 txg 24 那一版由记录施加出来、照候选走（`walk::versions_applied_only_by_records`），
//!   实例 1 那张实例表由它走到，丁-defer 门槛取什么都一样，那一形钉不住门槛。
//! - 影子账：先造一条被抛弃的根——实例 1 最新的根 txg 24 那次发布的系统配置轮换没落、下一次可写挂载时它的根槽与数据单元一时读不出，择根落到 23、
//!   实例 2 写行（行 (1, 23)），之后 24 读得出、按实例表被抛弃。再连崩到 txg 36 那一槽也被跳过（24 那一槽不被盖掉）、做完一次可写挂载，
//!   再挂时影子账按候选集算：实例 1 那张实例表被被抛弃的 24 引用，也被洞里实例 1 的有效根（在窗口外）引用，豁免、不隔离；
//!   照一圈窗口滤候选时窗口外那几条不豁免，那几槽被隔离。

mod common_root_ring_holes;

use common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    parameters_of_the_ring_hole_pools, pool_of, RingHoleDevices,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, raise_rollback_floor, Mounted, ShadowLedger};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    effective_rollback_floor, instance_table_of_root, readable_roots,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_format::{
    DATA_UNIT_BYTES, ROOT_RING_REGIONS, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use std::collections::{BTreeMap, BTreeSet};

/// 影子账那一形实例 1 覆盖写到的 txg（被抛弃的就是这一条）。
const LAST_TXG_OF_INSTANCE_ONE: u64 = 24;
/// 丁-defer 那一形实例 1 发到的 txg（被抛弃的那一条；它前一条 7 就是 B）。
const ABANDONED_TXG_UNDER_THE_RAISED_FLOOR: u64 = 8;
/// 影子账那一形造出被抛弃的根之后连崩的次数：跳过的 txg 要盖住 24 那一槽的下一圈 txg 36（24 + 12），不让它被盖掉。
const CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT: usize = 11;

/// 一圈窗口的长度（3S）。
fn ring_length_in_txgs() -> u64 {
    ROOT_RING_REGIONS * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS
}

/// 盘面上读得出的根、最新那条根指着的实例表、按它判被抛弃。
struct RootsOfTheRing {
    newest: RootRecord,
    valid: Vec<RootRecord>,
    abandoned: Vec<RootRecord>,
}

fn roots_of_the_ring(image: &MemoryPool) -> RootsOfTheRing {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let newest = choose_root(image, &system_configuration).expect("择得到根");
    let table = instance_table_of_root(image, &newest).expect("最新那条根的实例表读得出");
    let immutable = &system_configuration.immutable;
    let (abandoned, valid): (Vec<RootRecord>, Vec<RootRecord>) = readable_roots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .partition(|root| {
        table
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    });
    RootsOfTheRing {
        newest,
        valid,
        abandoned,
    }
}

/// 一条根在不在锚在 `anchor` 上的一圈窗口里（txg > N − 3S）。
fn inside_the_one_revolution_window(root: &RootRecord, anchor: CheckpointTxg) -> bool {
    root.checkpoint_txg.0 + ring_length_in_txgs() > anchor.0
}

/// 盘面上池级 checker 判 I-3.1 违例的那几块盘（名字加细节）。
fn allocated_statistic_violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(invariant, _)| *invariant == "I-3.1")
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 场景：实例 1 发到 txg 8，8 被崩溃恢复抛弃（实例 2 写行 (1, 7)，B = 7）；再挂一次（实例 3）覆盖写到 txg 18；下一次可写挂载崩在写行 txg 19
/// 那次根槽写之前（19 与 B 同槽：B 留在洞里）；再一次可写挂载做完；同一个进程里把 F 抬到 8。
/// 预期（前提）：B 还在环里、有效、在 F 之下、在锚在最新根上的一圈窗口外，窗口里最旧的有效根高于 F；
/// 池级 checker 不判 I-3.1——丁-defer 的门槛取整个环：max(8, 环里最旧有效根 7) = 8，B 引用、释放代高于 8 的槽并进遍历。
/// 判别力：丁-defer 门槛改回一圈窗口（`crates/mutations.tsv`「第10g批 丁-defer 门槛改回一圈窗口…」）时门槛是窗口里最旧的有效根，
/// 那几槽不并、记账多于遍历，最后一项红。
#[test]
fn raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring(
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let root_before_the_abandoned_one = ABANDONED_TXG_UNDER_THE_RAISED_FLOOR - 1;
    let floor = CheckpointTxg(ABANDONED_TXG_UNDER_THE_RAISED_FLOOR);
    let image =
        pool_with_the_newest_root_of_instance_one_abandoned(ABANDONED_TXG_UNDER_THE_RAISED_FLOOR);
    let image_before_the_hole = {
        let mut devices = devices_of(&image);
        let Mounted {
            output,
            mut allocator,
            current,
        } = mount_writable(&parameters, &mut devices).expect("被抛弃的根造出来之后再挂一次");
        let PoolVersion::WithFile(mut current) = current else {
            panic!("B 那一版带文件");
        };
        // 迭代上界：覆盖写到 B 那一槽下一圈的 txg 之前那一个；跨轮携带的是现行那一版。
        while current.root.checkpoint_txg.0 + 1
            < root_before_the_abandoned_one + ring_length_in_txgs()
        {
            let salt = current.root.checkpoint_txg.0 + 1;
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content_of_the_version(salt),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
                },
                output.instance,
            )
            .expect("覆盖写");
        }
        pool_of(&devices)
    };
    let (image_with_the_hole, row_txg_of_the_crashed_mount) =
        crash_a_mount_before_its_row_root(&image_before_the_hole);
    assert_eq!(
        row_txg_of_the_crashed_mount,
        root_before_the_abandoned_one + ring_length_in_txgs(),
        "前提：崩在写行之前的那次挂载要写的根槽就是 B 那一槽"
    );
    let mut devices = devices_of(&image_with_the_hole);
    let Mounted {
        mut allocator,
        current,
        ..
    } = mount_writable(&parameters, &mut devices).expect("洞之后的可写挂载做完");
    let PoolVersion::WithFile(mut current) = current else {
        panic!("现行那一版带文件");
    };
    raise_rollback_floor(
        &parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        floor,
        ShadowLedger::On,
    )
    .expect("F 抬到 8（不高于上限）");
    let image_after_the_raise = pool_of(&devices);
    let roots = roots_of_the_ring(&image_after_the_raise);
    let below_the_floor_outside_the_window: Vec<CheckpointTxg> = roots
        .valid
        .iter()
        .filter(|root| {
            root.checkpoint_txg < floor
                && !inside_the_one_revolution_window(root, roots.newest.checkpoint_txg)
        })
        .map(|root| root.checkpoint_txg)
        .collect();
    let oldest_valid_root_inside_the_window = roots
        .valid
        .iter()
        .filter(|root| inside_the_one_revolution_window(root, roots.newest.checkpoint_txg))
        .map(|root| root.checkpoint_txg)
        .min();
    assert!(
        below_the_floor_outside_the_window == vec![CheckpointTxg(root_before_the_abandoned_one)]
            && oldest_valid_root_inside_the_window.is_some_and(|oldest| oldest > floor),
        "前提：F 之下、一圈窗口外的有效根恰是 B {below_the_floor_outside_the_window:?}；窗口里最旧的有效根 {oldest_valid_root_inside_the_window:?} 高于 F（最新根 {:?}）",
        roots.newest.checkpoint_txg
    );
    assert_eq!(
        allocated_statistic_violations(&image_after_the_raise),
        Vec::<String>::new(),
        "丁-defer 门槛取整个环：B 引用、还在 defer 里的槽并进遍历，记账与遍历对得上"
    );
}

/// 实例 1 覆盖写到 `last` 那一刻的盘与它最后那次发布；另交最后那次发布之前每块盘两槽系统配置的字节（盘号、偏移、字节）。
struct PoolOfInstanceOne {
    devices: RingHoleDevices,
    last_publish: TransactionOutput,
    system_configuration_before_the_last_publish:
        Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)>,
}

/// 照 `common_root_ring_holes::pool_overwritten_through` 同一个做法（mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 txg `last`），
/// 多留最后那次发布与它之前的系统配置槽。
fn pool_of_instance_one_overwritten_through(last: u64) -> PoolOfInstanceOne {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    let mut system_configuration_before_the_last_publish = Vec::new();
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while current.root.checkpoint_txg.0 < last {
        let salt = current.root.checkpoint_txg.0 + 1;
        if salt == last {
            system_configuration_before_the_last_publish = system_configuration_slots_of(&devices);
        }
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            instance,
        )
        .expect("覆盖写");
    }
    PoolOfInstanceOne {
        devices,
        last_publish: current,
        system_configuration_before_the_last_publish,
    }
}

/// 每块盘两个系统配置槽的全部字节。
fn system_configuration_slots_of(
    devices: &RingHoleDevices,
) -> Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> {
    let spacing = u64::from(
        parameters_of_the_ring_hole_pools()
            .geometry
            .fixed_structure_slot_spacing,
    );
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut saved = Vec::new();
    // 迭代上界是盘数 × 每块盘两槽；没有提前出口。
    for (identity, device) in devices {
        for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = DeviceOffsetInBytes(slot * spacing);
            let mut bytes = vec![0u8; slot_bytes];
            device.read_at(offset, &mut bytes).expect("读系统配置槽");
            saved.push((*identity, offset, bytes));
        }
    }
    saved
}

fn write_bytes(
    devices: &mut RingHoleDevices,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    bytes: &[u8],
) {
    devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
        .write_at(offset, bytes, WriteDurability::Plain)
        .expect("写进内存盘");
}

/// 崩溃恢复造一条被抛弃的根（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 同一形，换到 12 槽小环上）：
/// 实例 1 最新那次发布（txg `last`）的系统配置轮换没落（每块盘两槽退回它之前的字节）；下一次可写挂载时它的根槽与数据单元一时读不出（清零），
/// 择根落到前一条根、它那条记录施加前验点名单元失败、不施加，实例 2 写行与暖机；挂载交回之后把它的根槽原样写回（数据单元不写回：那几槽实例 2 可能已经用了）。
/// 交回之后的盘面：txg `last` 的根读得出、按实例 2 的实例表被抛弃。
fn pool_with_the_newest_root_of_instance_one_abandoned(last: u64) -> MemoryPool {
    let parameters = parameters_of_the_ring_hole_pools();
    let PoolOfInstanceOne {
        mut devices,
        last_publish,
        system_configuration_before_the_last_publish,
    } = pool_of_instance_one_overwritten_through(last);
    for (device, offset, bytes) in &system_configuration_before_the_last_publish {
        write_bytes(&mut devices, *device, *offset, bytes);
    }
    let target = target_for_publish(
        last_publish.root.checkpoint_txg,
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let mut root_slot = vec![0u8; root_slot_bytes];
    devices
        .iter()
        .find(|(identity, _)| *identity == root_slot_device)
        .expect("池里有这块盘")
        .1
        .read_at(root_slot_offset, &mut root_slot)
        .expect("暂存最新那条根的根槽");
    write_bytes(
        &mut devices,
        root_slot_device,
        root_slot_offset,
        &vec![0u8; root_slot_bytes],
    );
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    for location in last_publish
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
    {
        write_bytes(
            &mut devices,
            location.device,
            location.slot.to_device_offset(),
            &vec![0u8; data_unit_bytes],
        );
    }
    let mounted =
        mount_writable(&parameters, &mut devices).expect("择根落到前一条根，照常可写挂载");
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(last - 1)),
        "前提：恢复落到 txg {} 那一版，实例 2",
        last - 1
    );
    write_bytes(&mut devices, root_slot_device, root_slot_offset, &root_slot);
    pool_of(&devices)
}

/// 场景：实例 1 覆盖写到 24，24 那次发布被崩溃恢复抛弃（实例 2 写行 (1, 23)）；再连崩 11 次在写行根槽之前（跳过的 txg 盖住 36，24 那一槽留着）、
/// 一次可写挂载做完；再可写挂载一次，看重建分配器影子账隔离了几槽。
/// 预期（前提）：盘上只有一条被抛弃的根 24；有被它引用、也只被锚在最新有效根上的一圈窗口外的有效根引用（窗口里的有效根与现行那一版的账都不引用）的槽。
/// 影子账按候选集（按实例表有效 ∧ txg ≥ F，不按窗口滤）豁免：每块盘隔离的槽数 = 被抛弃的根引用的槽里，候选根与现行那一版的账都不引用的那几槽的跨度之和。
/// 判别力：影子账的候选过滤改回一圈窗口（`crates/mutations.tsv`「第10g批 影子账候选过滤改回一圈窗口…」）时窗口外那几条根不豁免，
/// 它们与被抛弃的根同引用的槽被隔离，隔离数变多，最后一项红。
#[test]
fn a_remount_isolates_no_slot_an_abandoned_root_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window(
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut image = pool_with_the_newest_root_of_instance_one_abandoned(LAST_TXG_OF_INSTANCE_ONE);
    // 迭代上界：连崩的次数，每次一个洞；跨轮携带的是崩溃之后的盘面。
    for _ in 0..CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT {
        let (crashed, _row_txg) = crash_a_mount_before_its_row_root(&image);
        image = crashed;
    }
    let mut devices = devices_of(&image);
    mount_writable(&parameters, &mut devices).expect("连崩之后的可写挂载做完");
    let image_before_the_remount = pool_of(&devices);

    let roots = roots_of_the_ring(&image_before_the_remount);
    let effective_floor = {
        let system_configuration =
            choose_system_configuration(&image_before_the_remount).expect("系统配置");
        let immutable = system_configuration.immutable;
        effective_rollback_floor(
            &image_before_the_remount,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
        )
    };
    let start_slots_allocated_under = |root: &RootRecord| -> Vec<(DeviceIdentity, u64, u64)> {
        allocation_records_under_root(&image_before_the_remount, root)
            .expect("这一版的账读得出")
            .into_iter()
            .filter(|record| !record.is_released)
            .map(|record| (record.device, record.slot.0, u64::from(record.span_slots)))
            .collect()
    };
    let newest_valid_root = roots
        .valid
        .iter()
        .map(|root| root.checkpoint_txg)
        .max()
        .expect("有有效根");
    let candidates: Vec<&RootRecord> = roots
        .valid
        .iter()
        .filter(|root| root.checkpoint_txg >= effective_floor)
        .collect();
    let referenced_by = |roots_referencing: &[&RootRecord]| -> BTreeSet<(DeviceIdentity, u64)> {
        roots_referencing
            .iter()
            .flat_map(|root| start_slots_allocated_under(root))
            .map(|(device, slot, _)| (device, slot))
            .collect()
    };
    let current_version = choose_root(
        &image_before_the_remount,
        &choose_system_configuration(&image_before_the_remount).expect("系统配置"),
    )
    .expect("现行那一版的根");
    let allocated_in_the_current_version: BTreeSet<(DeviceIdentity, u64)> =
        referenced_by(&[&current_version]);
    let referenced_by_candidates = referenced_by(&candidates);
    let candidates_inside_the_window: Vec<&RootRecord> = candidates
        .iter()
        .copied()
        .filter(|root| inside_the_one_revolution_window(root, newest_valid_root))
        .collect();
    let referenced_by_candidates_inside_the_window = referenced_by(&candidates_inside_the_window);
    let mut expected_isolated_per_device: BTreeMap<DeviceIdentity, u64> = BTreeMap::new();
    let mut shared_only_with_candidates_outside_the_window: BTreeSet<(DeviceIdentity, u64)> =
        BTreeSet::new();
    let mut counted: BTreeSet<(DeviceIdentity, u64)> = BTreeSet::new();
    // 迭代上界是被抛弃的根的条数乘以它们账里的记录数；没有提前出口。
    for abandoned in &roots.abandoned {
        for (device, slot, span) in start_slots_allocated_under(abandoned) {
            let key = (device, slot);
            if allocated_in_the_current_version.contains(&key) {
                continue;
            }
            if referenced_by_candidates.contains(&key) {
                if !referenced_by_candidates_inside_the_window.contains(&key) {
                    shared_only_with_candidates_outside_the_window.insert(key);
                }
                continue;
            }
            if counted.insert(key) {
                *expected_isolated_per_device.entry(device).or_insert(0) += span;
            }
        }
    }
    assert!(
        roots.abandoned.iter().map(|root| (root.instance, root.checkpoint_txg)).collect::<Vec<_>>()
            == vec![(InstanceGeneration(1), CheckpointTxg(LAST_TXG_OF_INSTANCE_ONE))]
            && !shared_only_with_candidates_outside_the_window.is_empty(),
        "前提：只有被抛弃的根 txg 24；它与窗口外的有效根同引用、窗口里的有效根与现行那一版的账都不引用的槽 {shared_only_with_candidates_outside_the_window:?}（最新有效根 {newest_valid_root:?}，F {effective_floor:?}）"
    );

    let mut devices_of_the_remount = devices_of(&image_before_the_remount);
    let remounted = mount_writable(&parameters, &mut devices_of_the_remount).expect("再可写挂载");
    let isolated_per_device: BTreeMap<DeviceIdentity, u64> = remounted
        .allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .filter(|(_, isolated)| *isolated > 0)
        .collect();
    assert_eq!(
        isolated_per_device, expected_isolated_per_device,
        "影子账按整个环的候选集豁免：窗口外的有效根与被抛弃的根同引用的 {shared_only_with_candidates_outside_the_window:?} 不隔离"
    );
}

/// 探针：K1 那一形（实例 1 到 24 被抛弃，连崩 11 次，做完一次可写挂载，F 抬到 24），看 I-3.1 与槽的释放代。
#[test]
fn probe_k1_shape_raise_floor_to_24() {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut image = pool_with_the_newest_root_of_instance_one_abandoned(LAST_TXG_OF_INSTANCE_ONE);
    for _ in 0..CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT {
        let (crashed, _row_txg) = crash_a_mount_before_its_row_root(&image);
        image = crashed;
    }
    let mut devices = devices_of(&image);
    let Mounted {
        mut allocator,
        current,
        ..
    } = mount_writable(&parameters, &mut devices).expect("连崩之后的可写挂载做完");
    let PoolVersion::WithFile(mut current) = current else {
        panic!("现行那一版带文件");
    };
    let result = raise_rollback_floor(
        &parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        CheckpointTxg(20),
        ShadowLedger::On,
    );
    println!("PROBE raise result {:?}", result.as_ref().map(|_| ()).map_err(|e| format!("{e:?}")));
    let image_after = pool_of(&devices);
    let roots = roots_of_the_ring(&image_after);
    println!("PROBE newest {:?} valid {:?}", roots.newest.checkpoint_txg, roots.valid.iter().map(|r| r.checkpoint_txg.0).collect::<Vec<_>>());
    println!("PROBE I-3.1 violations {:?}", allocated_statistic_violations(&image_after));
}
