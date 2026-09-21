//! c381-r2 攻方副本：单段历史，按环境变量给参数，打印每一步的原样。
//! 例：`C381_PREFIX=base C381_ARM=5 C381_FAULTS=18:torn_err cargo test --release -p singlefs-harness --test c381r3_single -- --nocapture`
#[path = "c381r3_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;

fn snapshot_named(name: &str) -> Snapshot {
    let base = base_snapshot();
    match name {
        "base" => base,
        "txg6" => plus_empty_publishes(&base, 1, "txg6"),
        "txg7" => plus_empty_publishes(&base, 2, "txg7"),
        "remounted" => remounted(&base, "remounted"),
        "rolled_back" => rolled_back(&base, "rolled_back"),
        "many_overwrites" => plus_overwrites(&remounted(&base, "remounted"), 5, "many_overwrites"),
        // c381-r3：根环绕过一圈（R × S = 24 个根槽）之后，C 的根槽里装着 24 代之前那条根。
        "wrapped" => plus_empty_publishes(&base, 22, "wrapped"),
        "floor_raised" => {
            let many = plus_overwrites(&remounted(&base, "remounted"), 5, "many_overwrites");
            floor_raised(&many, "floor_raised")
        }
        other => panic!("没有这段前缀：{other}"),
    }
}

fn fault_named(name: &str) -> Fault {
    match name {
        "transient" => Fault::Transient,
        "lying" => Fault::Lying,
        "torn_err" => Fault::TornError,
        "torn_ok" => Fault::TornSilent,
        other => panic!("没有这种故障：{other}"),
    }
}

#[test]
fn single_history() {
    let parameters = e142_parameters(512, 512);
    let prefix = std::env::var("C381_PREFIX").unwrap_or_else(|_| "base".to_string());
    let snapshot = snapshot_named(&prefix);
    let arm: u8 = std::env::var("C381_ARM")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .expect("臂");
    let faults: Vec<(u64, Fault)> = std::env::var("C381_FAULTS")
        .unwrap_or_default()
        .split(',')
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (index, kind) = entry.split_once(':').expect("下标:故障");
            (index.parse().expect("下标"), fault_named(kind))
        })
        .collect();
    let actions: Vec<Action> = std::env::var("C381_ACTIONS")
        .unwrap_or_default()
        .split(',')
        .filter(|entry| !entry.is_empty())
        .map(|entry| match entry {
            "Retry" => Action::Retry,
            "Empty" => Action::Empty,
            "Other" => Action::Other,
            "Remount" => Action::Remount,
            other => panic!("没有这个动作：{other}"),
        })
        .collect();
    let spec = Spec {
        faults,
        barrier_fault: std::env::var("C381_BARRIER").ok().map(|value| value.parse().expect("屏障下标")),
        dead_device: std::env::var("C381_DEAD").ok().map(|value| {
            let (device, from) = value.split_once(':').expect("盘:下标");
            (device.parse().expect("盘"), from.parse().expect("下标"))
        }),
        crash_at: std::env::var("C381_CRASH").ok().map(|value| value.parse().expect("断电下标")),
        crash_torn: std::env::var("C381_CRASH_TORN").is_ok(),
        actions,
        lying_ranges: std::env::var("C381_LYING")
            .ok()
            .iter()
            .flat_map(|value| {
                value.split(';').map(|entry| {
                    let mut parts = entry.split(':');
                    let device = parts.next().expect("盘").parse().expect("盘");
                    let start: u64 = parts.next().expect("起").parse().expect("起");
                    let length: u64 = parts.next().expect("长").parse().expect("长");
                    (device, start, start + length)
                })
            })
            .collect(),
        probe_to_device_zero: std::env::var("C381_PROBE0").is_ok(),
        probe_mode: match std::env::var("C381_PROBE_MODE").unwrap_or_else(|_| "fixed".to_string()).as_str() {
            "fixed" => ProbeMode::Fixed,
            "failed" => ProbeMode::FailedLanding,
            "retry" => ProbeMode::FailedLandingRetry,
            other => panic!("没有这种探针读法：{other}"),
        },
        rollback_probe: std::env::var("C381_ROLLBACK").ok().map(|value| {
            let (instance, txg) = value.split_once(':').expect("实例:txg");
            (instance.parse().expect("实例"), txg.parse().expect("txg"))
        }),
        switch_selects_base_root: arm == 6,
        bad_ranges: std::env::var("C381_BAD")
            .ok()
            .iter()
            .flat_map(|value| {
                value.split(';').map(|entry| {
                    let mut parts = entry.split(':');
                    let device = parts.next().expect("盘").parse().expect("盘");
                    let start: u64 = parts.next().expect("起").parse().expect("起");
                    let length: u64 = parts.next().expect("长").parse().expect("长");
                    (device, start, start + length)
                })
            })
            .collect(),
    };
    println!("[前缀 {}] {}", snapshot.label, snapshot.note);
    println!("[spec] {spec:?}");
    let outcome = run(&parameters, &snapshot, arm, &spec);
    println!(
        "[{}] 分类={}\n  steps={:?}\n  writes_after_c={} writes={} 切换={} 只读={} 实例={} 探针={}\n  checker@crash={:?}\n  remount={}\n  checker@after={:?}\n  内容={}",
        arm_name(arm),
        classify(&outcome),
        outcome.steps,
        outcome.writes_after_c,
        outcome.writes,
        outcome.switches,
        outcome.read_only,
        outcome.final_instance,
        outcome.probe_writes,
        outcome.checker_at_crash,
        outcome.remount,
        outcome.checker_after_remount,
        outcome.acknowledged_content_after_remount
    );
    println!(
        "  探针落点={:?}\n  探针写抹掉={:?}\n  落点没有定义次数={}\n  下次挂载之前的盘面={:?}\n  回退试挂={}",
        outcome.probe_notes,
        outcome.destroyed,
        outcome.probe_undefined,
        outcome.census_before_remount,
        outcome.rollback_result
    );
}
