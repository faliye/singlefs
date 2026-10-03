//! 攻方腿 m3-formal-code-r1 提的改法（只在拷贝里，被攻过零轮）：经胶水 `choose_root` 钉「txg 为主」——
//! 前一个空槽写实例 1、txg = mkfs + 2，后一个空槽写实例 2、txg = mkfs + 1；D22 已定项 7 txg 为主 ⇒ 择实例 1 那条。
//! 胶水把择新键两项摊反（实例代号为主）时择到实例 2 那条。
mod common;

use common::format_pool;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading};
use singlefs_core::root_ring::slot_offset;

#[test]
fn a_later_checkpoint_txg_in_an_earlier_slot_beats_a_higher_instance_through_choose_root() {
    let mut image = format_pool("attack-root-choice-txg-beats-instance").memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let mkfs_root = choose_root(&image, &system_configuration).expect("mkfs 根");
    let immutable = system_configuration.immutable;
    let empty: Vec<_> = every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter(|ring_slot| {
            matches!(
                read_root_ring_slot(&image, &immutable.region_devices, &immutable.sizes, &immutable.filesystem_identifier, *ring_slot),
                RootRingSlotReading::Bad(_)
            )
        })
        .collect();
    let (earlier, later) = (empty[0], *empty.last().expect("有空槽"));
    let root_slot_bytes = usize::try_from(immutable.sizes.physical_block_size).expect("槽宽");
    let mut newer_txg = mkfs_root;
    newer_txg.instance = InstanceGeneration(1);
    newer_txg.checkpoint_txg = CheckpointTxg(mkfs_root.checkpoint_txg.0 + 2);
    let mut higher_instance = mkfs_root;
    higher_instance.instance = InstanceGeneration(2);
    higher_instance.checkpoint_txg = CheckpointTxg(mkfs_root.checkpoint_txg.0 + 1);
    for (ring_slot, root) in [(earlier, newer_txg), (later, higher_instance)] {
        let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域")];
        image.devices.get_mut(&device).expect("盘").write(
            slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
            &root.to_slot(root_slot_bytes),
        );
    }
    let chosen = choose_root(&image, &system_configuration).expect("择得出");
    assert_eq!(chosen, newer_txg, "txg 为主：txg 大的实例 1 那条赢过实例代号高、txg 小的实例 2 那条");
}
