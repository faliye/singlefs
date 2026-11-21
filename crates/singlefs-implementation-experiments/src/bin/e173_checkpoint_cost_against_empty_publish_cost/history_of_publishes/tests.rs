//! `history_of_publishes` 的单测。
use super::*;

/// 单测走的短历史：G-1（两块 1 GiB）、S = 4，顺序写 3 个单元、覆盖写 2 次、卸载再挂一圈。与产物走同一份 [`run_the_history`]，
/// 只把长度缩短（登记第九节「执行员先在单测里核这一格在原式与变异上确实不同」）。
fn short_history() -> HistoryReport {
    run_the_history(
        Geometry {
            slots_per_region: 4,
            pool_size: PoolSize::OneGibibyte,
        },
        HistoryPlan {
            sequentially_written_data_units: 3,
            overwrites_per_cycle: 2,
            cycles: 1,
        },
    )
}

#[test]
fn the_short_history_completes_and_the_local_kappa_equals_the_core_kappa_on_every_publish() {
    let report = short_history();
    assert_eq!(report.end, HistoryEnd::Completed, "短历史走完");
    let readings: Vec<CheckpointCostReading> = report
        .publishes
        .iter()
        .filter_map(|observation| observation.checkpoint_cost)
        .chain(report.final_checkpoint_cost)
        .collect();
    assert!(
        readings
            .iter()
            .filter(|reading| reading.built_on == BuiltOn::VersionWithTrees)
            .count()
            >= 8,
        "短历史里接在有树那几版后面的发布至少 8 次：{readings:?}"
    );
    for reading in &readings {
        assert_eq!(
            reading.core,
            reading.local.total(),
            "停机条款 S2：本地 κ（{}）等于核心层 κ",
            reading.local.terms_text()
        );
    }
}

#[test]
fn both_routes_agree_on_every_device_on_every_obtained_publish_of_a_short_history() {
    let report = short_history();
    let compared: Vec<&PublishObservation> = report
        .publishes
        .iter()
        .filter(|observation| {
            observation
                .fixed_point
                .as_ref()
                .is_some_and(|account| account.by_records_per_device.is_some())
        })
        .collect();
    for kind in [
        PublishKind::SequentialWrite,
        PublishKind::Overwrite,
        PublishKind::UnmountChain,
        PublishKind::WarmUpOfTheMount,
    ] {
        assert!(
            compared
                .iter()
                .filter(|observation| observation.kind == kind)
                .count()
                >= 1,
            "两条路都取得出的发布里有 {} 那一种",
            kind.label()
        );
    }
    for observation in compared {
        let account = observation.fixed_point.as_ref().expect("只留了取得出的");
        assert_eq!(
            account.routes_agree_on_every_device(),
            Some(true),
            "停机条款 S3：txg {} 的 {} 两条路逐盘相同：路一 {}、路二 {}",
            observation.txg,
            observation.kind.label(),
            account.by_roles,
            account.by_records_text()
        );
    }
}

#[test]
fn the_route_doubling_hook_turns_the_route_check_red_on_the_first_overwrite_of_a_short_history() {
    let report = short_history();
    let control = report
        .positive_control_of_the_routes
        .expect("短历史有覆盖写，PC7 有读数");
    assert_eq!(
        control.hook_on_routes_agree,
        Some(false),
        "PC7：钩子开着时 S3 判红"
    );
    assert_eq!(
        control.hook_off_routes_agree,
        Some(true),
        "PC7：钩子关着时两条路相同"
    );
    assert!(control.ordinary_slots > 0, "PC7：覆盖写的普通分配槽数 > 0");
    assert!(
        positive_control_row_and_holds(&report).1,
        "PC7 那一行 holds=true"
    );
}

#[test]
fn every_empty_publish_of_a_short_history_writes_no_ordinary_slot_and_no_more_than_kappa() {
    let report = short_history();
    let mut empty_publishes_checked = 0;
    for observation in report
        .publishes
        .iter()
        .filter(|observation| observation.kind.is_an_empty_publish())
    {
        let Some(account) = &observation.fixed_point else {
            continue;
        };
        assert_eq!(
            account.ordinary_slots, 0,
            "txg {} 的空发布没有普通分配",
            observation.txg
        );
        if let (Some(actual), Some(cost)) = (
            observation.actual_empty_publish_cost(),
            observation.checkpoint_cost,
        ) {
            assert!(
                actual <= cost.core,
                "A6：txg {} 的空发布新占 {actual} 槽不超过之前那一版的 κ {}",
                observation.txg,
                cost.core
            );
            empty_publishes_checked += 1;
        }
    }
    assert!(
        empty_publishes_checked >= 3,
        "短历史里取得出的空发布至少 3 次（卸载那一串 2 或 3 次、再挂的暖机）"
    );
}
