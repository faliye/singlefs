//! 装置自己的独立解码（不借 `singlefs_core::recovery` 的实例表判时间线）与 S5 那一项：独立解码与 `singlefs_core::recovery` 的输出逐条比。

use super::*;

// ============================================================================
// 五、装置自己的独立解码：不借 `singlefs_core::recovery` 的实例表判时间线，
//    走 `singlefs_checker::image`（另一套解析代码，D13 已定项 8 那条「checker 独立判」的现成落点）。
// ============================================================================

/// 两盘系统配置解出的几何一致才交回；否则报 S5（两边都查）。
pub(super) fn independent_geometry(pool: &MemoryPool) -> Result<PoolGeometry, String> {
    let chosen = checker_image::chosen_system_configurations(pool);
    let mut geometries = Vec::new();
    for (device, entry) in &chosen {
        match entry {
            Some((_, geometry)) => geometries.push(*geometry),
            None => return Err(format!("S5: 设备 {device} 没有自证的系统配置")),
        }
    }
    let first = *geometries
        .first()
        .ok_or_else(|| "S5: 池里没有盘".to_string())?;
    for other in &geometries[1..] {
        if other != &first {
            return Err("S5: 两盘系统配置解出的几何不一致".to_string());
        }
    }
    Ok(first)
}

/// 根环里全部自证过的根，按 (实例, txg)（checker 的独立解码，`singlefs_checker::check_root_slot`）。
pub(super) fn readable_roots_independent(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
) -> BTreeSet<TimelineRoot> {
    checker_image::valid_roots(pool, geometry)
        .into_iter()
        .map(|(_, _, view)| (view.instance, view.checkpoint_txg))
        .collect()
}

/// 根环里全部自证过的根，连 (区域, 槽) 一起（PC3 要按 (区域, 槽) 点名故障落点）。
pub(super) fn readable_roots_with_slots(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
) -> Vec<(u64, u64, TimelineRoot)> {
    checker_image::valid_roots(pool, geometry)
        .into_iter()
        .map(|(region, slot, view)| (region, slot, (view.instance, view.checkpoint_txg)))
        .collect()
}

/// 这一步「还有可读被抛弃根的回退事件数」（N_live，Q3-2 的 N_w 取它全族的最大值）与
/// 「读得出的被抛弃根条数」（去重后的根数）。纯函数、不碰 `crates/`：PC-Nw 直接单测它。
pub(super) fn count_live_rollback_events(
    rollback_events: &[RollbackEvent],
    readable: &BTreeSet<TimelineRoot>,
) -> (u64, u64) {
    let mut events_live = 0u64;
    let mut roots_live: BTreeSet<TimelineRoot> = BTreeSet::new();
    for event in rollback_events {
        let mut any = false;
        for root in &event.abandoned {
            if readable.contains(root) {
                roots_live.insert(*root);
                any = true;
            }
        }
        if any {
            events_live += 1;
        }
    }
    (
        events_live,
        u64::try_from(roots_live.len()).expect("根数装得进 u64"),
    )
}

pub(super) fn live_rollback_events(
    node: &SimNode,
    readable: &BTreeSet<TimelineRoot>,
) -> (u64, u64) {
    count_live_rollback_events(&node.rollback_events, readable)
}

// ============================================================================
// 八、S5：装置自己的解码（checker 独立实现）与 `singlefs_core::recovery` 的输出逐条比。
// ============================================================================

pub(super) fn run_independent_decode_cross_check() -> bool {
    let geometry = GEOMETRY_PRIMARY;
    let node = match bootstrap(&geometry, 2) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!("name=s5_independent_decode_cross_check verdict=fail reason=\"bootstrap 失败：{error}\""));
            return false;
        }
    };
    let devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let system_configuration = match singlefs_core::recovery::choose_system_configuration(&devices)
    {
        Ok(system_configuration) => system_configuration,
        Err(error) => {
            emit_result(&format!(
                "name=s5_independent_decode_cross_check verdict=fail reason=\"crates::recovery::choose_system_configuration 报错：{error:?}\""
            ));
            return false;
        }
    };
    let crates_readable: BTreeSet<TimelineRoot> = singlefs_core::recovery::readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root_pair(&root))
    .collect();
    let independent_geometry_view = match independent_geometry(&node.pool) {
        Ok(geometry_view) => geometry_view,
        Err(error) => {
            emit_result(&format!(
                "name=s5_independent_decode_cross_check verdict=fail reason={error:?}"
            ));
            return false;
        }
    };
    let device_readable = readable_roots_independent(&node.pool, &independent_geometry_view);
    if crates_readable == device_readable {
        emit_result(&format!(
            "name=s5_independent_decode_cross_check verdict=pass readable_root_count={} readable_roots={:?}",
            crates_readable.len(),
            crates_readable
        ));
        true
    } else {
        emit_result(&format!(
            "name=s5_independent_decode_cross_check verdict=fail crates_readable={crates_readable:?} device_readable={device_readable:?}"
        ));
        false
    }
}

#[cfg(test)]
mod tests;
