//! litmus 与实现绑在一起：`litmus/first-txn-*.litmus` 里发布一侧（P0）的写与屏障次序，
//! 必须就是第一个事务录制流里「单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA」那一段的次序。
//! herd7 判的是 litmus 写下的那个形态；这条测试保证那个形态就是代码今天发出的形态——
//! 改了发布顺序而没改 litmus，或者改了 litmus 而代码没跟，这里判红。

mod common;

use common::{build_pool, geometry};
use singlefs_harness::segments::StepKind;

fn writer_steps_of(litmus_file_name: &str) -> Vec<StepKind> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../litmus")
        .join(litmus_file_name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读不了 {}：{error}", path.display()));
    let writer = text
        .split("P0(")
        .nth(1)
        .expect("litmus 里有 P0")
        .split("\n}")
        .next()
        .expect("P0 有函数体");
    writer
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("smp_wmb()") {
                Some(StepKind::Barrier)
            } else if line.starts_with("WRITE_ONCE(*unit") {
                Some(StepKind::UnitWrite)
            } else if line.starts_with("WRITE_ONCE(*journal") {
                Some(StepKind::JournalRecord)
            } else if line.starts_with("WRITE_ONCE(*root") {
                Some(StepKind::RootRecordFua)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn litmus_writer_threads_follow_the_recorded_publish_order() {
    let pool = build_pool("litmus-binding");
    let operations = pool.stream.operations();
    let transaction = &operations[operations.len() - 23..];
    let mut collapsed: Vec<StepKind> = Vec::new();
    for operation in transaction {
        let kind = geometry().classify(operation);
        if kind == StepKind::SuperblockSlot || collapsed.last() == Some(&kind) {
            continue;
        }
        collapsed.push(kind);
    }
    assert_eq!(
        collapsed,
        vec![
            StepKind::UnitWrite,
            StepKind::Barrier,
            StepKind::JournalRecord,
            StepKind::Barrier,
            StepKind::RootRecordFua
        ],
        "第一个事务的发布顺序（D16（发布语义） 已定项 7）"
    );
    for never in [
        "first-txn-root-implies-units.litmus",
        "first-txn-journal-implies-units.litmus",
    ] {
        assert_eq!(
            writer_steps_of(never),
            collapsed,
            "{never} 的 P0 要与录制流的发布顺序逐项相同"
        );
    }
    // 对照组是同一个写者去掉屏障：写的次序不变，屏障少了。
    for (control, barriers_left) in [
        ("first-txn-root-implies-units-nofence.litmus", 0usize),
        ("first-txn-journal-implies-units-nofence.litmus", 1),
    ] {
        let steps = writer_steps_of(control);
        let writes: Vec<StepKind> = steps
            .iter()
            .copied()
            .filter(|step| *step != StepKind::Barrier)
            .collect();
        assert_eq!(
            writes,
            vec![
                StepKind::UnitWrite,
                StepKind::JournalRecord,
                StepKind::RootRecordFua
            ],
            "{control} 写的次序与 Never 那一条相同"
        );
        assert_eq!(
            steps
                .iter()
                .filter(|step| **step == StepKind::Barrier)
                .count(),
            barriers_left,
            "{control} 剩几道屏障"
        );
    }
}
