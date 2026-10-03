//! 攻方腿副本探针（m3-leftover-code-r1-opus，K3）：同一段历史在今天的判法（甲）与出路 B 的草图上各给什么结局。
//! 形 H4：两次覆盖写 B、C 之后进程退出；下一次可写挂载时 C 的根槽、C 的数据单元与每块盘见证 C 的那一槽系统配置都读回全 0
//! （设备一层的暂时读错，读回 0；harness 的 `UnreadableRangeReadBack::Zeros` 那一形），读完之后照读。
mod common;
mod common_tree_split;

use common::{file_content, parameters};
use common_tree_split::TreeSplitPool;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::CodeTwoTreeNodeCapacities;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};

#[derive(Clone, Copy, Debug)]
enum WitnessSlotReading {
    ReadsBackZeros,
    DeviceError,
}

fn run_h4(zero_witness_on: &[u32]) -> String {
    let modes: Vec<(u32, WitnessSlotReading)> = zero_witness_on.iter().map(|device| (*device, WitnessSlotReading::ReadsBackZeros)).collect();
    run_h4_with(&modes)
}

fn run_h4_with(witness_modes: &[(u32, WitnessSlotReading)]) -> String {
    let mut pool = TreeSplitPool::with_the_first_file_version_under(
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let mut second = file_content();
    second[0] ^= 1;
    pool.overwrite(&second, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    let mut third = file_content();
    third[1] ^= 1;
    pool.overwrite(&third, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    let newest = pool.output.clone();
    let publish_parameters = parameters();
    let spacing = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let mut devices = pool.reopened_devices();
    let target = singlefs_core::root_ring::target_for_publish(
        newest.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_device = publish_parameters.region_devices[usize::try_from(target.region).expect("区域")];
    let root_offset = singlefs_core::root_ring::slot_offset(target, publish_parameters.geometry.fixed_structure_slot_spacing);
    let root_bytes = usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let mut hidden: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> = vec![(root_device, root_offset, root_bytes)];
    for pointer in &newest.data_pointers {
        for location in pointer.locations {
            hidden.push((location.device, location.slot.to_device_offset(), usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768")));
        }
    }
    let mut erroring: Vec<common::UnreadableRange> = Vec::new();
    for (device, mode) in witness_modes {
        let identity = DeviceIdentity(*device);
        let mut slots = singlefs_core::recovery::verified_system_configuration_slots(
            devices.as_slice(),
            identity,
            spacing,
            &publish_parameters.filesystem_identifier,
        );
        slots.sort_by_key(|slot| slot.quantities.slot_generation);
        let newest_generation = slots.last().expect("两槽").quantities.slot_generation;
        let offset = (newest_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing;
        match mode {
            WitnessSlotReading::ReadsBackZeros => hidden.push((
                identity,
                DeviceOffsetInBytes(offset),
                usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
            )),
            WitnessSlotReading::DeviceError => erroring.push(common::UnreadableRange {
                device: identity,
                offset_in_bytes: offset,
                length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
                failing_reads: common::FailingReadsOfARange::Every,
            }),
        }
    }
    let mut saved = Vec::new();
    for (identity, offset, length) in &hidden {
        let device = &mut devices.iter_mut().find(|(candidate, _)| candidate == identity).expect("盘").1;
        let mut bytes = vec![0u8; *length];
        device.read_at(*offset, &mut bytes).expect("暂存");
        device.wrapped_device_mut().image.write(*offset, &vec![0u8; *length]);
        saved.push((*identity, *offset, bytes));
    }
    let ranges = common::SharedUnreadableRanges::new(erroring, common::UnreadableRangeReadBack::DeviceError);
    let mut wrapped = common::with_unreadable_ranges(devices, &ranges);
    let outcome = mount_writable(&publish_parameters, &mut wrapped);
    let mut devices = common::without_unreadable_ranges(wrapped);
    for (identity, offset, bytes) in &saved {
        let device = &mut devices.iter_mut().find(|(candidate, _)| candidate == identity).expect("盘").1;
        device.wrapped_device_mut().image.write(*offset, bytes);
    }
    match outcome {
        Ok(mounted) => format!(
            "MOUNTED newest_before=({},{}) effective_root=({},{}) new_instance={} rows={:?}",
            newest.root.instance.0,
            newest.root.checkpoint_txg.0,
            mounted.output.effective_root.instance.0,
            mounted.output.effective_root.checkpoint_txg.0,
            mounted.output.instance.0,
            mounted.output.rows_written
        ),
        Err(error) => format!("REFUSED {error:?}"),
    }
}

#[test]
fn opus_probe_k3_h4_witness_slots_read_back_zero_with_the_newest_root() {
    for zero_witness_on in [vec![0u32, 1], vec![0u32], vec![]] {
        let line = run_h4(&zero_witness_on);
        eprintln!("name=opus-k3-h4 zero_witness_on={zero_witness_on:?} {}", &line[..line.len().min(700)]);
    }
    use WitnessSlotReading::{DeviceError, ReadsBackZeros};
    for modes in [
        vec![(0u32, DeviceError), (1, DeviceError)],
        vec![(0u32, DeviceError)],
        vec![(0u32, ReadsBackZeros), (1, DeviceError)],
    ] {
        let line = run_h4_with(&modes);
        eprintln!("name=opus-k3-h4 witness_modes={modes:?} {}", &line[..line.len().min(700)]);
    }
}
