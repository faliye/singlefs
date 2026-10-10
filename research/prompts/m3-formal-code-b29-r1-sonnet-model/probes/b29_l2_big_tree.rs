//! b29 正推腿 L2 探针：inode 树长到三片叶容器的合法镜像（T13 形状的历史），整份录制流施加之后跑池级 checker，数新判法的判定。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation,
    HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

fn run(label: &str, operations: Vec<HistoryOperation>) {
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations };
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::FourGibibytes, space_admission: SpaceAdmission::JudgedByTheFormula },
        &stream,
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{label}");
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], HistoryDeviceWidth::FourGibibytes.device_bytes());
    pool.apply(&stream.retained_operations());
    let verdicts = check_pool_image(&pool);
    let pick = |name: &str| verdicts.iter().find(|(n, _)| *n == name).map(|(_, v)| match v {
        InvariantVerdict::Holds => "Holds".to_string(),
        InvariantVerdict::Violated(t) => format!("Violated({})", t.chars().take(80).collect::<String>()),
        InvariantVerdict::NotApplicable(t) => format!("NotApplicable({})", t.chars().take(50).collect::<String>()),
        InvariantVerdict::PartiallyJudged(t) => format!("PartiallyJudged({})", t.chars().take(80).collect::<String>()),
    }).unwrap();
    let violated: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect();
    println!("BIGTREE {label} violated={violated:?} I-9.16={} I-5.4={} I-2.4={} I-5.5={}", pick("I-9.16"), pick("I-5.4"), pick("I-2.4"), pick("I-5.5"));
}

#[test]
fn b29_l2_three_leaf_containers() {
    let mut a = vec![HistoryOperation::Unlink];
    a.extend(std::iter::repeat_n(HistoryOperation::CreateInode, 470));
    run("470_creates_three_containers", a);
    let mut b = vec![HistoryOperation::Unlink];
    b.extend(std::iter::repeat_n(HistoryOperation::CreateInode, 468));
    b.push(HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 2 }));
    b.extend(std::iter::repeat_n(HistoryOperation::CreateInode, 3));
    run("T13_rollback_past_the_third_container", b);
    let mut c = vec![HistoryOperation::Unlink];
    c.extend(std::iter::repeat_n(HistoryOperation::CreateInode, 705));
    run("705_creates_four_containers", c);
}
