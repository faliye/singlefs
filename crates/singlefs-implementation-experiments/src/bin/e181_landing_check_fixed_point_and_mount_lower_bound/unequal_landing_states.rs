//! 不等宽几何上的落点读数与它的那一段。

use super::*;

/// 一个状态上各盘最低落点是否不等，以及这一个单元落得下那一判放不放行（登记 5.3 节 Q9）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct UnequalLandingReading {
    pub(super) lowest_slot_per_device: Vec<Option<u64>>,
    pub(super) lowest_slots_differ: bool,
    pub(super) one_data_unit_lands: bool,
}

pub(super) fn unequal_landing_reading(pool: &Pool) -> UnequalLandingReading {
    let cluster_segments = pool.allocator.cluster_segments();
    let lowest_slot_per_device: Vec<Option<u64>> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            device_map
                .lowest_user_data_slot(cluster_segments)
                .map(|slot| slot.0)
        })
        .collect();
    let lowest_slots_differ = lowest_slot_per_device
        .iter()
        .any(|slot| *slot != lowest_slot_per_device[0]);
    let checkpoint_cost = core_checkpoint_cost(
        &pool.current,
        &pool.allocator,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let one_data_unit_lands = admit_the_units_landing_on_every_device(
        &PlacementRoomOnOneDevice::of_every_device_of_the_allocator(&pool.allocator),
        UnitsOfAPublishToLand {
            data_units: 1,
            commit_generated_slots: NEW_INODE_ORDINARY_COMMIT_GENERATED_SLOTS + checkpoint_cost,
        },
    )
    .is_ok();
    UnequalLandingReading {
        lowest_slot_per_device,
        lowest_slots_differ,
        one_data_unit_lands,
    }
}

/// 登记 5.3 节 Q9 与它的阳性对照：G不等大上从第一个文件起换写到小盘段外槽对数的 25%、50%、75%，每个状态判一次；
/// 对照把小盘单元区填到末尾（照 `unequal_devices.rs` 那一形，只改空闲图），各盘最低落点必须不等。
pub(super) fn run_unequal_landing_states(
    content: &mut ContentBuffer,
) -> Result<(u64, u64, bool), String> {
    let mut base = build_pool_after_the_first_file(
        GEOMETRY_UNEQUAL,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let smaller_pairs = data_unit_pairs_outside_the_clusters(&base, 1);
    let mut states = 0;
    let mut unequal_and_landing = 0;
    let mut readings = vec![(
        "after_the_first_file".to_string(),
        unequal_landing_reading(&base),
    )];
    for percent in [25_u64, 50, 75] {
        let mut fork = base.fork();
        let units = usize::try_from(smaller_pairs * percent / 100).expect("单元数");
        match write_units_and_account(
            &mut fork,
            units,
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            content,
        )? {
            WriteOutcome::Published(_) => {
                readings.push((
                    format!("filled_{percent}_percent_of_the_smaller_device"),
                    unequal_landing_reading(&fork),
                ));
            }
            WriteOutcome::Refused(refusal) => {
                emit(&format!(
                    "name=per_device_lowest_slot_unequal_fill_refused percent={percent} units={units} refusal={refusal:?}"
                ));
            }
        }
    }
    for (state, reading) in &readings {
        states += 1;
        unequal_and_landing +=
            u64::from(reading.lowest_slots_differ && reading.one_data_unit_lands);
        emit(&format!(
            "name=per_device_lowest_slot_unequal_state geometry={} state={state} lowest_slot_per_device={:?} lowest_slots_differ={} one_data_unit_lands={}",
            GEOMETRY_UNEQUAL.name,
            reading.lowest_slot_per_device,
            reading.lowest_slots_differ,
            reading.one_data_unit_lands
        ));
    }
    let mut control = base.fork();
    let smaller_end = control.allocator.devices[1].unit_area_start().slot().0
        + control.allocator.devices[1].unit_area_slots();
    for device_map in &mut control.allocator.devices {
        let first = device_map.unit_area_start().slot().0;
        for slot in first..smaller_end {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    let control_reading = unequal_landing_reading(&control);
    emit(&format!(
        "name=per_device_lowest_slot_unequal_positive_control lowest_slot_per_device={:?} lowest_slots_differ={} one_data_unit_lands={} control_hits={}",
        control_reading.lowest_slot_per_device,
        control_reading.lowest_slots_differ,
        control_reading.one_data_unit_lands,
        control_reading.lowest_slots_differ
    ));
    Ok((
        states,
        unequal_and_landing,
        control_reading.lowest_slots_differ,
    ))
}
