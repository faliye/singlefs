//! 抬 F 的上限按每一条根所在那一槽的区域认盘（D16（发布语义） 已定项 1「抬 F 的上限」那一行字面「每块盘上最新的持久有效根」；
//! 判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节「Q3 外带　第 0 代根记在哪块盘」：写者原来按落点公式从 txg 推区域，
//! mkfs 在三个区域的槽 0 各种的那一份第 0 代根 txg 都是 0，三份都记在区域 0 的盘上；池级 checker 按根所在那一槽记）。
//!
//! 盘面照攻方那段历史的根环（`research/prompts/m3-close-b19-reason-r2-opus-output.md` 第四节「第 3 问外带」那张表：R = 3、S = 4、
//! 区域 1 独占盘 1、区域 1 每一版都崩在根槽之前）：真写者在环洞池上 mkfs、可写挂载（零单元的发布 1、2）、写第一个文件（txg 3），
//! 之后 txg 除以 3 余 1 的那几版（4、7、10、13）覆盖写、其余空发布，推到 txg 14；再把盘 1 上区域 1 的四个根槽换成那段历史里的样子——
//! 槽 0 是 mkfs 写下的第 0 代根（从 mkfs 刚做完的盘上原样抄来），槽 1 到 3 全零（那几版的根槽都没落）。区域 0 与区域 2 的八个根槽、
//! 单元与记录都是真写者写的。这一刻环里的有效根（F = 0）：第 0 代根（盘 1）与 3、5、6、8、9、11、12、14（盘 0）；
//! 用户可见的不同状态从新到旧是 14、11、8、5、3，第 4 新的是 5。盘 1 上最新的有效根是第 0 代根，上限 = min(14, 0, 5) = 0。
//! 按落点公式认盘时第 0 代根记在盘 0 上、盘 1 一条都没有，上限算成 min(14, 5) = 5——写者会把 F 抬到 5、越过盘 1 上那唯一一份根。
//! 这一形在真写者的可写挂载里走不走得到没有核：可写挂载的暖机推到每块盘上都有本实例的根，这里只在算上限的式子上钉。
mod common_root_ring_holes;

use std::collections::BTreeSet;

use common_root_ring_holes::{
    content_of_the_version, devices_of, parameters_of_the_ring_hole_pools, pool_of, publish_empty,
    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, rollback_floor_ceiling, Mounted};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, readable_roots_with_ring_slots,
};
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

/// 盘 1 独占的区域。
const REGION_OF_DEVICE_ONE: u64 = 1;

/// 推到的最后一版：攻方表里准入抬 F 之前那一刻环里最新的几条根都在盘 0 上。
const LAST_TXG_OF_THE_HISTORY: u64 = 14;

/// 用户可见的不同状态从新到旧 14、11、8、5、3 里第 4 新的那一条：按落点公式认盘时算出的上限。
const FOURTH_NEWEST_DISTINCT_STATE: CheckpointTxg = CheckpointTxg(5);

/// 那段历史推到 txg 14 之后的盘面，区域 1 的四个根槽换成攻方表里的样子；交回盘面与现行那一版。
fn ring_of_the_history_with_region_one_crashing_before_every_root_slot(
) -> (MemoryPool, TransactionOutput) {
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
    let slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let offset_of = |slot: u64| {
        slot_offset(
            RootRingSlot {
                region: REGION_OF_DEVICE_ONE,
                slot,
            },
            parameters.geometry.fixed_structure_slot_spacing,
        )
    };
    let device_one =
        parameters.region_devices[usize::try_from(REGION_OF_DEVICE_ONE).expect("区域号")];
    let genesis_root_bytes_on_device_one =
        pool_of(&devices).devices[&device_one].read(offset_of(0), slot_bytes);
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("只做过 mkfs 的池可写挂载");
    assert_eq!(
        current.root().checkpoint_txg,
        CheckpointTxg(2),
        "前提：只做过 mkfs 的池可写挂载推零单元的发布 1、2，到每块盘上都有本实例的根"
    );
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
    // 迭代上界：txg 逐次加一推到 14；跨轮携带的是现行那一版。区域 1 的那几版（txg 除以 3 余 1）改用户可见状态，其余空发布。
    while current.root.checkpoint_txg.0 < LAST_TXG_OF_THE_HISTORY {
        let txg = current.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = if txg % 3 == REGION_OF_DEVICE_ONE {
            publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content_of_the_version(txg),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + txg,
                },
                instance,
            )
            .expect("覆盖写")
        } else {
            publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布")
        };
    }
    let mut image = pool_of(&devices);
    let device_one_image = image.devices.get_mut(&device_one).expect("盘 1 在池里");
    device_one_image.write(offset_of(0), &genesis_root_bytes_on_device_one);
    // 迭代上界：区域 1 的槽 1 到 S − 1。
    for slot in 1..parameters.geometry.root_ring_slots_per_region.count() {
        device_one_image.zero_fill(
            offset_of(slot),
            u64::from(parameters.geometry.physical_block_size),
        );
    }
    (image, current)
}

/// 场景：攻方那段历史的根环——盘 1 上只有区域 1 槽 0 那一份第 0 代根，区域 1 别的槽全零；盘 0 上 3 到 14 的八条根，F = 0。
/// 预期：盘 1 上读得出的有效根只有第 0 代根（txg 0），抬 F 的上限是 0，不越过它。
/// 判别力：写者按落点公式从 txg 推区域认盘时，第 0 代根三份都记在盘 0 上，盘 1 一条有效根都没有、不进取最小，上限算成 5，这一条红
/// （`crates/mutations.tsv`「Q3 外带 写者算每块盘上最新有效根改回按落点公式从 txg 认盘」）。
#[test]
fn admission_floor_ceiling_stops_at_the_genesis_root_left_alone_on_device_one_in_its_ring_slot() {
    let (image, current) = ring_of_the_history_with_region_one_crashing_before_every_root_slot();
    let system_configuration = choose_system_configuration(&image).expect("系统配置读得出");
    let immutable = &system_configuration.immutable;
    let roots_on_region_one: Vec<(u64, CheckpointTxg)> = readable_roots_with_ring_slots(
        &image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|(ring_slot, _)| ring_slot.region == REGION_OF_DEVICE_ONE)
    .map(|(ring_slot, root)| (ring_slot.slot, root.checkpoint_txg))
    .collect();
    assert_eq!(
        roots_on_region_one,
        vec![(0, CheckpointTxg(0))],
        "前提：区域 1（盘 1）上读得出的根只有槽 0 那一份第 0 代根"
    );
    let table = instance_table_chain_of_root(&image, &current.root)
        .expect("现行那一版的实例表读得出")
        .records;
    let ceiling = rollback_floor_ceiling(
        &devices_of(&image),
        &system_configuration,
        CheckpointTxg(0),
        &table,
        &BTreeSet::new(),
    )
    .expect("算得出上限：环里有效根的树表都读得出");
    assert_ne!(
        ceiling, FOURTH_NEWEST_DISTINCT_STATE,
        "按落点公式认盘时上限是第 4 新的不同状态 5：第 0 代根那一份算到了盘 0 上"
    );
    assert_eq!(
        ceiling,
        CheckpointTxg(0),
        "盘 1 上最新的有效根是它槽里的第 0 代根，上限不越过它"
    );
}
