//! litmus 与实现绑在一起：`litmus/new-pool-file-creation-*.litmus` 里发布一侧（P0）的写与屏障次序，
//! 必须就是新池新建文件录制流里「单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA」那一段的次序。
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
    // 暖机与新池新建文件的分界取自 warm_up_operation_count（暖机已调完、发布还没调），不从流尾往回数几步：
    // 写死的步数过期时事务前几次单元写会被划进暖机，`collapsed` 却因为去重相邻同类而看不出来——
    // 下面这条步数断言才是钉住它的那一条（里程碑「覆盖写、释放、回退与复用」收尾批「55 号装置步数」）。
    let transaction = &operations[pool.warm_up_operation_count..];
    assert_eq!(
        transaction.len(),
        35,
        "新池新建文件的步数写死错了就在这里先红：24 次单元写 + 2 道屏障 + 2 条 journal 记录 + 2 道屏障 \
         + 1 次根槽 FUA + 2 次系统配置槽写 + 2 道屏障（C577：轮换之后、发布返回之前），与 new_pool_file_creation_publish.rs 的断言\
         `(35, \"24+2+1+2\", 16_777_223)` 同一个数"
    );
    // litmus 的写者线程只写到根槽；根槽之后的系统配置槽轮换与它之后那道池屏障（C577）另钉。
    let root_position = transaction
        .iter()
        .rposition(|operation| geometry().classify(operation) == StepKind::RootRecordFua)
        .expect("新池新建文件写了根槽");
    assert_eq!(
        transaction[root_position + 1..]
            .iter()
            .map(|operation| geometry().classify(operation))
            .collect::<Vec<StepKind>>(),
        vec![
            StepKind::SystemConfigurationSlot,
            StepKind::SystemConfigurationSlot,
            StepKind::Barrier,
            StepKind::Barrier
        ],
        "根槽之后：两块盘各一次系统配置槽轮换，再一道池屏障（两块盘各一步）才返回"
    );
    let mut collapsed: Vec<StepKind> = Vec::new();
    for operation in &transaction[..=root_position] {
        let kind = geometry().classify(operation);
        if kind == StepKind::SystemConfigurationSlot || collapsed.last() == Some(&kind) {
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
        "新池新建文件的发布顺序（D16（发布语义） 已定项 7）"
    );
    for never in [
        "new-pool-file-creation-root-implies-units.litmus",
        "new-pool-file-creation-journal-implies-units.litmus",
    ] {
        assert_eq!(
            writer_steps_of(never),
            collapsed,
            "{never} 的 P0 要与录制流的发布顺序逐项相同"
        );
    }
    // 对照组是同一个写者去掉屏障：写的次序不变，屏障少了。
    for (control, barriers_left) in [
        (
            "new-pool-file-creation-root-implies-units-nofence.litmus",
            0usize,
        ),
        (
            "new-pool-file-creation-journal-implies-units-nofence.litmus",
            1,
        ),
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
